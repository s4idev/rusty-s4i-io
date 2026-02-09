//! HTTP/HTTPS transport implementation

use crate::error::{Error, Result};
use crate::transport::{TransportConfig, TransportEvent, TransportId, TransportService, TransportType};
use async_trait::async_trait;
use bytes::Bytes;
use std::collections::VecDeque;
use tokio::sync::Mutex;
use std::sync::Arc;

/// HTTP transport implementation
pub struct HttpTransport {
    config: TransportConfig,
    #[cfg(feature = "http")]
    client: Option<reqwest::Client>,
    events: Arc<Mutex<VecDeque<TransportEvent>>>,
    connected: Arc<Mutex<bool>>,
}

impl HttpTransport {
    /// Create a new HTTP transport
    pub fn new(config: TransportConfig) -> Result<Self> {
        if !matches!(config.transport_type, TransportType::Http | TransportType::Https) {
            return Err(Error::Configuration(
                "Invalid transport type for HTTP".to_string(),
            ));
        }

        Ok(Self {
            config,
            #[cfg(feature = "http")]
            client: None,
            events: Arc::new(Mutex::new(VecDeque::new())),
            connected: Arc::new(Mutex::new(false)),
        })
    }
}

#[async_trait]
impl TransportService for HttpTransport {
    async fn connect(&mut self) -> Result<()> {
        #[cfg(feature = "http")]
        {
            self.client = Some(reqwest::Client::new());
            *self.connected.lock().await = true;
            self.events.lock().await.push_back(TransportEvent::Connected(
                TransportId::Connection(0)
            ));
            Ok(())
        }
        #[cfg(not(feature = "http"))]
        {
            Err(Error::NotSupported("HTTP feature not enabled".to_string()))
        }
    }

    async fn disconnect(&mut self) -> Result<()> {
        #[cfg(feature = "http")]
        {
            self.client = None;
        }
        *self.connected.lock().await = false;
        self.events.lock().await.push_back(TransportEvent::Disconnected(
            TransportId::Connection(0)
        ));
        Ok(())
    }

    async fn send(&mut self, _id: &TransportId, data: Bytes) -> Result<()> {
        #[cfg(feature = "http")]
        {
            if let Some(client) = &self.client {
                let url = if self.config.transport_type == TransportType::Https {
                    format!("https://{}", self.config.address)
                } else {
                    format!("http://{}", self.config.address)
                };
                
                client.post(&url)
                    .body(data.to_vec())
                    .send()
                    .await
                    .map_err(|e| Error::Connection(e.to_string()))?;
                
                Ok(())
            } else {
                Err(Error::Connection("Not connected".to_string()))
            }
        }
        #[cfg(not(feature = "http"))]
        {
            let _ = data;
            Err(Error::NotSupported("HTTP feature not enabled".to_string()))
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
        self.config.transport_type.clone()
    }
}

#[cfg(all(test, feature = "http"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_http_transport_creation() {
        let config = TransportConfig {
            transport_type: TransportType::Http,
            address: "example.com".to_string(),
            is_server: false,
            options: Default::default(),
        };
        let transport = HttpTransport::new(config);
        assert!(transport.is_ok());
    }

    #[tokio::test]
    async fn test_http_connect() {
        let config = TransportConfig {
            transport_type: TransportType::Http,
            address: "example.com".to_string(),
            is_server: false,
            options: Default::default(),
        };
        let mut transport = HttpTransport::new(config).unwrap();
        let result = transport.connect().await;
        assert!(result.is_ok());
        assert!(transport.is_connected());
    }
}
