//! WebSocket transport implementation

use crate::error::{Error, Result};
use crate::transport::{TransportConfig, TransportEvent, TransportId, TransportService, TransportType};
use async_trait::async_trait;
use bytes::Bytes;
use std::collections::VecDeque;
use tokio::sync::Mutex;
use std::sync::Arc;

/// WebSocket transport implementation
pub struct WebSocketTransport {
    config: TransportConfig,
    events: Arc<Mutex<VecDeque<TransportEvent>>>,
    connected: Arc<Mutex<bool>>,
}

impl WebSocketTransport {
    /// Create a new WebSocket transport
    pub fn new(config: TransportConfig) -> Result<Self> {
        if config.transport_type != TransportType::WebSocket {
            return Err(Error::Configuration(
                "Invalid transport type for WebSocket".to_string(),
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
impl TransportService for WebSocketTransport {
    async fn connect(&mut self) -> Result<()> {
        #[cfg(feature = "websocket")]
        {
            // WebSocket connection implementation would go here
            *self.connected.lock().await = true;
            self.events.lock().await.push_back(TransportEvent::Connected(
                TransportId::Connection(0)
            ));
            Ok(())
        }
        #[cfg(not(feature = "websocket"))]
        {
            Err(Error::NotSupported("WebSocket feature not enabled".to_string()))
        }
    }

    async fn disconnect(&mut self) -> Result<()> {
        *self.connected.lock().await = false;
        self.events.lock().await.push_back(TransportEvent::Disconnected(
            TransportId::Connection(0)
        ));
        Ok(())
    }

    async fn send(&mut self, _id: &TransportId, _data: Bytes) -> Result<()> {
        #[cfg(feature = "websocket")]
        {
            if *self.connected.lock().await {
                // WebSocket send implementation would go here
                Ok(())
            } else {
                Err(Error::Connection("Not connected".to_string()))
            }
        }
        #[cfg(not(feature = "websocket"))]
        {
            Err(Error::NotSupported("WebSocket feature not enabled".to_string()))
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
        TransportType::WebSocket
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_transport_creation() {
        let config = TransportConfig {
            transport_type: TransportType::WebSocket,
            address: "localhost:8080".to_string(),
            is_server: false,
            options: Default::default(),
        };
        let transport = WebSocketTransport::new(config);
        assert!(transport.is_ok());
    }
}
