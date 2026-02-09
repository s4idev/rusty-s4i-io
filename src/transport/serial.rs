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

/// Serial port transport implementation
pub struct SerialTransport {
    #[allow(dead_code)]
    config: TransportConfig,
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
            // Serial port connection implementation would go here
            *self.connected.lock().await = true;
            self.events
                .lock()
                .await
                .push_back(TransportEvent::Connected(TransportId::Connection(0)));
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
        *self.connected.lock().await = false;
        self.events
            .lock()
            .await
            .push_back(TransportEvent::Disconnected(TransportId::Connection(0)));
        Ok(())
    }

    async fn send(&mut self, _id: &TransportId, _data: Bytes) -> Result<()> {
        #[cfg(feature = "serial")]
        {
            if *self.connected.lock().await {
                // Serial send implementation would go here
                Ok(())
            } else {
                Err(Error::Connection("Not connected".to_string()))
            }
        }
        #[cfg(not(feature = "serial"))]
        {
            Err(Error::NotSupported(
                "Serial feature not enabled".to_string(),
            ))
        }
    }

    async fn poll_event(&mut self) -> Result<Option<TransportEvent>> {
        let mut events = self.events.lock().await;
        Ok(events.pop_front())
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
