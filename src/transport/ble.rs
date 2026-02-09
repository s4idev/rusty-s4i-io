//! Bluetooth Low Energy (BLE) transport implementation

use crate::error::{Error, Result};
use crate::transport::{
    TransportConfig, TransportEvent, TransportId, TransportService, TransportType,
};
use async_trait::async_trait;
use bytes::Bytes;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(feature = "ble")]
use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter, WriteType};
#[cfg(feature = "ble")]
use btleplug::platform::{Adapter, Manager, Peripheral};
#[cfg(feature = "ble")]
use std::time::Duration;
#[cfg(feature = "ble")]
use uuid::Uuid;

/// BLE transport implementation
pub struct BleTransport {
    config: TransportConfig,
    #[cfg(feature = "ble")]
    peripheral: Arc<Mutex<Option<Peripheral>>>,
    #[cfg(feature = "ble")]
    service_uuid: Option<Uuid>,
    events: Arc<Mutex<VecDeque<TransportEvent>>>,
    connected: Arc<Mutex<bool>>,
}

impl BleTransport {
    /// Create a new BLE transport
    pub fn new(config: TransportConfig) -> Result<Self> {
        if config.transport_type != TransportType::Ble {
            return Err(Error::Configuration(
                "Invalid transport type for BLE".to_string(),
            ));
        }

        #[cfg(feature = "ble")]
        let service_uuid = if let Some(ref uuid_str) = config.options.service_uuid {
            Some(Uuid::parse_str(uuid_str)
                .map_err(|e| Error::Configuration(format!("Invalid service UUID: {}", e)))?)
        } else {
            None
        };

        Ok(Self {
            config,
            #[cfg(feature = "ble")]
            peripheral: Arc::new(Mutex::new(None)),
            #[cfg(feature = "ble")]
            service_uuid,
            events: Arc::new(Mutex::new(VecDeque::new())),
            connected: Arc::new(Mutex::new(false)),
        })
    }
}

#[async_trait]
impl TransportService for BleTransport {
    async fn connect(&mut self) -> Result<()> {
        #[cfg(feature = "ble")]
        {
            // Get the BLE adapter
            let manager = Manager::new()
                .await
                .map_err(|e| Error::Connection(format!("Failed to create BLE manager: {}", e)))?;

            let adapters = manager.adapters()
                .await
                .map_err(|e| Error::Connection(format!("Failed to get BLE adapters: {}", e)))?;

            let adapter = adapters
                .into_iter()
                .next()
                .ok_or_else(|| Error::Connection("No BLE adapter found".to_string()))?;

            // Start scanning
            adapter
                .start_scan(ScanFilter::default())
                .await
                .map_err(|e| Error::Connection(format!("Failed to start BLE scan: {}", e)))?;

            // Wait for devices to be discovered
            tokio::time::sleep(Duration::from_secs(2)).await;

            // Find peripheral by address (stored in config.address)
            let peripherals = adapter
                .peripherals()
                .await
                .map_err(|e| Error::Connection(format!("Failed to get peripherals: {}", e)))?;

            let peripheral = peripherals
                .into_iter()
                .find(|p| {
                    if let Ok(Some(props)) = p.properties().as_ref().ok().and_then(|p| p.as_ref()) {
                        props.local_name.as_ref().map_or(false, |name| name.contains(&self.config.address))
                    } else {
                        false
                    }
                })
                .ok_or_else(|| Error::Connection(format!("BLE device not found: {}", self.config.address)))?;

            // Connect to the peripheral
            peripheral
                .connect()
                .await
                .map_err(|e| Error::Connection(format!("Failed to connect to BLE device: {}", e)))?;

            // Discover services
            peripheral
                .discover_services()
                .await
                .map_err(|e| Error::Connection(format!("Failed to discover services: {}", e)))?;

            *self.peripheral.lock().await = Some(peripheral);
            *self.connected.lock().await = true;
            
            self.events
                .lock()
                .await
                .push_back(TransportEvent::Connected(TransportId::Connection(0)));
            
            log::info!("BLE connected to {}", self.config.address);
            Ok(())
        }
        #[cfg(not(feature = "ble"))]
        {
            Err(Error::NotSupported("BLE feature not enabled".to_string()))
        }
    }

    async fn disconnect(&mut self) -> Result<()> {
        #[cfg(feature = "ble")]
        {
            let mut peripheral_guard = self.peripheral.lock().await;
            if let Some(peripheral) = peripheral_guard.take() {
                let _ = peripheral.disconnect().await;
            }
        }
        
        *self.connected.lock().await = false;
        self.events
            .lock()
            .await
            .push_back(TransportEvent::Disconnected(TransportId::Connection(0)));
        
        log::info!("BLE disconnected");
        Ok(())
    }

    async fn send(&mut self, _id: &TransportId, data: Bytes) -> Result<()> {
        #[cfg(feature = "ble")]
        {
            let peripheral_guard = self.peripheral.lock().await;
            if let Some(peripheral) = peripheral_guard.as_ref() {
                // Find the first writable characteristic in the service
                let characteristics = peripheral.characteristics();
                
                let characteristic = if let Some(service_uuid) = self.service_uuid {
                    characteristics
                        .iter()
                        .find(|c| c.service_uuid == service_uuid && c.properties.write)
                        .ok_or_else(|| Error::Connection("No writable characteristic found".to_string()))?
                } else {
                    characteristics
                        .iter()
                        .find(|c| c.properties.write)
                        .ok_or_else(|| Error::Connection("No writable characteristic found".to_string()))?
                };

                peripheral
                    .write(characteristic, &data, WriteType::WithoutResponse)
                    .await
                    .map_err(|e| Error::Connection(format!("BLE write failed: {}", e)))?;

                Ok(())
            } else {
                Err(Error::Connection("Not connected".to_string()))
            }
        }
        #[cfg(not(feature = "ble"))]
        {
            let _ = data;
            Err(Error::NotSupported("BLE feature not enabled".to_string()))
        }
    }

    async fn poll_event(&mut self) -> Result<Option<TransportEvent>> {
        // First, check if there are any queued events
        let mut events = self.events.lock().await;
        if let Some(event) = events.pop_front() {
            return Ok(Some(event));
        }
        drop(events);

        #[cfg(feature = "ble")]
        {
            // BLE notifications/indications would need to be set up separately
            // For now, we just check connection status
            let peripheral_guard = self.peripheral.lock().await;
            if let Some(peripheral) = peripheral_guard.as_ref() {
                match peripheral.is_connected().await {
                    Ok(true) => Ok(None),
                    Ok(false) => {
                        drop(peripheral_guard);
                        let _ = self.disconnect().await;
                        Ok(Some(TransportEvent::Disconnected(TransportId::Connection(0))))
                    }
                    Err(_) => Ok(None),
                }
            } else {
                Ok(None)
            }
        }
        #[cfg(not(feature = "ble"))]
        {
            Ok(None)
        }
    }

    fn is_connected(&self) -> bool {
        self.connected.try_lock().map(|g| *g).unwrap_or(false)
    }

    fn transport_type(&self) -> TransportType {
        TransportType::Ble
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ble_transport_creation() {
        let config = TransportConfig {
            transport_type: TransportType::Ble,
            address: "AA:BB:CC:DD:EE:FF".to_string(),
            is_server: false,
            options: Default::default(),
        };
        let transport = BleTransport::new(config);
        assert!(transport.is_ok());
    }
}
