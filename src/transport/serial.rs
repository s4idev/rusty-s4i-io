//! Serial port transport implementation

use crate::error::{Error, Result};
use crate::transport::{
    TransportConfig, TransportEvent, TransportId, TransportService, TransportType,
};
use async_trait::async_trait;
use bytes::Bytes;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(feature = "serial")]
use tokio::io::{AsyncReadExt, AsyncWriteExt};
#[cfg(feature = "serial")]
use tokio_serial::SerialPortBuilderExt;

/// Serial port transport implementation
pub struct SerialTransport {
    config: TransportConfig,
    #[cfg(feature = "serial")]
    port: Arc<Mutex<Option<tokio_serial::SerialStream>>>,
    events: Arc<Mutex<VecDeque<TransportEvent>>>,
    connected: Arc<Mutex<bool>>,
}

impl SerialTransport {
    /// Create a new Serial transport
    pub fn new(config: TransportConfig) -> Result<Self> {
        if config.transport_type != TransportType::Serial {
            return Err(Error::Configuration(
                "Invalid transport type for Serial".to_string(),
            ));
        }

        Ok(Self {
            config,
            #[cfg(feature = "serial")]
            port: Arc::new(Mutex::new(None)),
            events: Arc::new(Mutex::new(VecDeque::new())),
            connected: Arc::new(Mutex::new(false)),
        })
    }
}

#[async_trait]
impl TransportService for SerialTransport {
    async fn connect(&mut self) -> Result<()> {
        #[cfg(feature = "serial")]
        {
            let baud_rate = self.config.options.baud_rate.unwrap_or(9600);
            
            let port = tokio_serial::new(&self.config.address, baud_rate)
                .open_native_async()
                .map_err(|e| Error::Connection(format!("Failed to open serial port: {}", e)))?;

            *self.port.lock().await = Some(port);
            *self.connected.lock().await = true;
            
            self.events
                .lock()
                .await
                .push_back(TransportEvent::Connected(TransportId::Connection(0)));
            
            log::info!("Serial port connected: {} at {} baud", self.config.address, baud_rate);
            Ok(())
        }
        #[cfg(not(feature = "serial"))]
        {
            Err(Error::NotSupported(
                "Serial feature not enabled".to_string(),
            ))
        }
    }

    async fn disconnect(&mut self) -> Result<()> {
        #[cfg(feature = "serial")]
        {
            *self.port.lock().await = None;
        }
        
        *self.connected.lock().await = false;
        self.events
            .lock()
            .await
            .push_back(TransportEvent::Disconnected(TransportId::Connection(0)));
        
        log::info!("Serial port disconnected");
        Ok(())
    }

    async fn send(&mut self, _id: &TransportId, data: Bytes) -> Result<()> {
        #[cfg(feature = "serial")]
        {
            let mut port_guard = self.port.lock().await;
            if let Some(port) = port_guard.as_mut() {
                port.write_all(&data).await?;
                port.flush().await?;
                Ok(())
            } else {
                Err(Error::Connection("Not connected".to_string()))
            }
        }
        #[cfg(not(feature = "serial"))]
        {
            let _ = data;
            Err(Error::NotSupported(
                "Serial feature not enabled".to_string(),
            ))
        }
    }

    async fn poll_event(&mut self) -> Result<Option<TransportEvent>> {
        // First, check if there are any queued events
        let mut events = self.events.lock().await;
        if let Some(event) = events.pop_front() {
            return Ok(Some(event));
        }
        drop(events);

        #[cfg(feature = "serial")]
        {
            // Try to read data from serial port
            let mut port_guard = self.port.lock().await;
            if let Some(port) = port_guard.as_mut() {
                let mut buffer = vec![0u8; 4096];

                // Non-blocking read with timeout
                match tokio::time::timeout(
                    std::time::Duration::from_millis(10),
                    port.read(&mut buffer),
                )
                .await
                {
                    Ok(Ok(0)) => {
                        // No data available or port closed
                        Ok(None)
                    }
                    Ok(Ok(n)) => {
                        let data = Bytes::copy_from_slice(&buffer[..n]);
                        Ok(Some(TransportEvent::DataReceived(
                            TransportId::Connection(0),
                            data,
                        )))
                    }
                    Ok(Err(e)) => {
                        Ok(Some(TransportEvent::Error(
                            TransportId::Connection(0),
                            e.to_string(),
                        )))
                    }
                    Err(_) => {
                        // Timeout - no data available
                        Ok(None)
                    }
                }
            } else {
                Ok(None)
            }
        }
        #[cfg(not(feature = "serial"))]
        {
            Ok(None)
        }
    }

    fn is_connected(&self) -> bool {
        self.connected.try_lock().map(|g| *g).unwrap_or(false)
    }

    fn transport_type(&self) -> TransportType {
        TransportType::Serial
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serial_transport_creation() {
        let config = TransportConfig {
            transport_type: TransportType::Serial,
            address: "/dev/ttyUSB0".to_string(),
            is_server: false,
            options: Default::default(),
        };
        let transport = SerialTransport::new(config);
        assert!(transport.is_ok());
    }
}
