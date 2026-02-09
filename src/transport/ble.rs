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

/// BLE transport implementation
pub struct BleTransport {
    #[allow(dead_code)]
    config: TransportConfig,
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

        Ok(Self {
            config,
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
            // BLE connection implementation would go here
            *self.connected.lock().await = true;
            self.events
                .lock()
                .await
                .push_back(TransportEvent::Connected(TransportId::Connection(0)));
            Ok(())
        }
        #[cfg(not(feature = "ble"))]
        {
            Err(Error::NotSupported("BLE feature not enabled".to_string()))
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
        #[cfg(feature = "ble")]
        {
            if *self.connected.lock().await {
                // BLE send implementation would go here
                Ok(())
            } else {
                Err(Error::Connection("Not connected".to_string()))
            }
        }
        #[cfg(not(feature = "ble"))]
        {
            Err(Error::NotSupported("BLE feature not enabled".to_string()))
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
