//! USB HID transport implementation

use crate::error::{Error, Result};
use crate::transport::{
    TransportConfig, TransportEvent, TransportId, TransportService, TransportType,
};
use async_trait::async_trait;
use bytes::Bytes;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(feature = "usb")]
use hidapi::HidApi;

/// USB HID transport implementation
pub struct UsbTransport {
    config: TransportConfig,
    #[cfg(feature = "usb")]
    device: Arc<Mutex<Option<hidapi::HidDevice>>>,
    events: Arc<Mutex<VecDeque<TransportEvent>>>,
    connected: Arc<Mutex<bool>>,
}

impl UsbTransport {
    /// Create a new USB HID transport
    pub fn new(config: TransportConfig) -> Result<Self> {
        if config.transport_type != TransportType::UsbHid {
            return Err(Error::Configuration(
                "Invalid transport type for USB HID".to_string(),
            ));
        }

        Ok(Self {
            config,
            #[cfg(feature = "usb")]
            device: Arc::new(Mutex::new(None)),
            events: Arc::new(Mutex::new(VecDeque::new())),
            connected: Arc::new(Mutex::new(false)),
        })
    }
}

#[async_trait]
impl TransportService for UsbTransport {
    async fn connect(&mut self) -> Result<()> {
        #[cfg(feature = "usb")]
        {
            let vendor_id = self.config.options.vendor_id
                .ok_or_else(|| Error::Configuration("Vendor ID required for USB HID".to_string()))?;
            let product_id = self.config.options.product_id
                .ok_or_else(|| Error::Configuration("Product ID required for USB HID".to_string()))?;

            // Initialize HID API
            let api = HidApi::new()
                .map_err(|e| Error::Connection(format!("Failed to initialize HID API: {}", e)))?;

            // Open device
            let device = api
                .open(vendor_id, product_id)
                .map_err(|e| Error::Connection(format!("Failed to open USB device: {}", e)))?;

            // Set non-blocking mode
            device
                .set_blocking_mode(false)
                .map_err(|e| Error::Connection(format!("Failed to set non-blocking mode: {}", e)))?;

            *self.device.lock().await = Some(device);
            *self.connected.lock().await = true;
            
            self.events
                .lock()
                .await
                .push_back(TransportEvent::Connected(TransportId::Connection(0)));
            
            log::info!("USB HID connected: VID={:04x}, PID={:04x}", vendor_id, product_id);
            Ok(())
        }
        #[cfg(not(feature = "usb"))]
        {
            Err(Error::NotSupported("USB feature not enabled".to_string()))
        }
    }

    async fn disconnect(&mut self) -> Result<()> {
        #[cfg(feature = "usb")]
        {
            *self.device.lock().await = None;
        }
        
        *self.connected.lock().await = false;
        self.events
            .lock()
            .await
            .push_back(TransportEvent::Disconnected(TransportId::Connection(0)));
        
        log::info!("USB HID disconnected");
        Ok(())
    }

    async fn send(&mut self, _id: &TransportId, data: Bytes) -> Result<()> {
        #[cfg(feature = "usb")]
        {
            let device_guard = self.device.lock().await;
            if let Some(device) = device_guard.as_ref() {
                device
                    .write(&data)
                    .map_err(|e| Error::Connection(format!("USB write failed: {}", e)))?;
                Ok(())
            } else {
                Err(Error::Connection("Not connected".to_string()))
            }
        }
        #[cfg(not(feature = "usb"))]
        {
            let _ = data;
            Err(Error::NotSupported("USB feature not enabled".to_string()))
        }
    }

    async fn poll_event(&mut self) -> Result<Option<TransportEvent>> {
        // First, check if there are any queued events
        let mut events = self.events.lock().await;
        if let Some(event) = events.pop_front() {
            return Ok(Some(event));
        }
        drop(events);

        #[cfg(feature = "usb")]
        {
            // Try to read data from USB device
            let device_guard = self.device.lock().await;
            if let Some(device) = device_guard.as_ref() {
                let mut buffer = vec![0u8; 64]; // Standard HID report size

                match device.read(&mut buffer) {
                    Ok(0) => {
                        // No data available
                        Ok(None)
                    }
                    Ok(n) => {
                        let data = Bytes::copy_from_slice(&buffer[..n]);
                        Ok(Some(TransportEvent::DataReceived(
                            TransportId::Connection(0),
                            data,
                        )))
                    }
                    Err(e) => {
                        Ok(Some(TransportEvent::Error(
                            TransportId::Connection(0),
                            e.to_string(),
                        )))
                    }
                }
            } else {
                Ok(None)
            }
        }
        #[cfg(not(feature = "usb"))]
        {
            Ok(None)
        }
    }

    fn is_connected(&self) -> bool {
        self.connected.try_lock().map(|g| *g).unwrap_or(false)
    }

    fn transport_type(&self) -> TransportType {
        TransportType::UsbHid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usb_transport_creation() {
        let config = TransportConfig {
            transport_type: TransportType::UsbHid,
            address: "vid:1234:pid:5678".to_string(),
            is_server: false,
            options: Default::default(),
        };
        let transport = UsbTransport::new(config);
        assert!(transport.is_ok());
    }
}
