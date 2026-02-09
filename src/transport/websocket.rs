//! WebSocket transport implementation

use crate::error::{Error, Result};
use crate::transport::{
    TransportConfig, TransportEvent, TransportId, TransportService, TransportType,
};
use async_trait::async_trait;
use bytes::Bytes;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(feature = "websocket")]
use futures_util::{SinkExt, StreamExt};
#[cfg(feature = "websocket")]
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream};

/// WebSocket transport implementation
pub struct WebSocketTransport {
    config: TransportConfig,
    #[cfg(feature = "websocket")]
    ws_stream: Arc<Mutex<Option<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>>>>,
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
            #[cfg(feature = "websocket")]
            ws_stream: Arc::new(Mutex::new(None)),
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
            // Build WebSocket URL
            let url = if self.config.address.starts_with("ws://") || self.config.address.starts_with("wss://") {
                self.config.address.clone()
            } else {
                format!("ws://{}", self.config.address)
            };

            // Connect to WebSocket server
            let (ws_stream, _) = connect_async(&url)
                .await
                .map_err(|e| Error::Connection(format!("WebSocket connection failed: {}", e)))?;

            *self.ws_stream.lock().await = Some(ws_stream);
            *self.connected.lock().await = true;
            
            self.events
                .lock()
                .await
                .push_back(TransportEvent::Connected(TransportId::Connection(0)));

            log::info!("WebSocket connected to {}", url);
            Ok(())
        }
        #[cfg(not(feature = "websocket"))]
        {
            Err(Error::NotSupported(
                "WebSocket feature not enabled".to_string(),
            ))
        }
    }

    async fn disconnect(&mut self) -> Result<()> {
        #[cfg(feature = "websocket")]
        {
            let mut ws_guard = self.ws_stream.lock().await;
            if let Some(mut ws) = ws_guard.take() {
                let _ = ws.close(None).await;
            }
        }
        
        *self.connected.lock().await = false;
        self.events
            .lock()
            .await
            .push_back(TransportEvent::Disconnected(TransportId::Connection(0)));
        
        log::info!("WebSocket disconnected");
        Ok(())
    }

    async fn send(&mut self, _id: &TransportId, data: Bytes) -> Result<()> {
        #[cfg(feature = "websocket")]
        {
            let mut ws_guard = self.ws_stream.lock().await;
            if let Some(ws) = ws_guard.as_mut() {
                ws.send(Message::Binary(data.to_vec()))
                    .await
                    .map_err(|e| Error::Connection(format!("WebSocket send failed: {}", e)))?;
                Ok(())
            } else {
                Err(Error::Connection("Not connected".to_string()))
            }
        }
        #[cfg(not(feature = "websocket"))]
        {
            let _ = data;
            Err(Error::NotSupported(
                "WebSocket feature not enabled".to_string(),
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

        #[cfg(feature = "websocket")]
        {
            // Try to read data from WebSocket
            let mut ws_guard = self.ws_stream.lock().await;
            if let Some(ws) = ws_guard.as_mut() {
                // Non-blocking read with timeout
                match tokio::time::timeout(
                    std::time::Duration::from_millis(10),
                    ws.next(),
                )
                .await
                {
                    Ok(Some(Ok(msg))) => {
                        match msg {
                            Message::Binary(data) => {
                                Ok(Some(TransportEvent::DataReceived(
                                    TransportId::Connection(0),
                                    Bytes::from(data),
                                )))
                            }
                            Message::Text(text) => {
                                Ok(Some(TransportEvent::DataReceived(
                                    TransportId::Connection(0),
                                    Bytes::from(text.into_bytes()),
                                )))
                            }
                            Message::Close(_) => {
                                drop(ws_guard);
                                let _ = self.disconnect().await;
                                Ok(Some(TransportEvent::Disconnected(TransportId::Connection(0))))
                            }
                            Message::Ping(_) | Message::Pong(_) => Ok(None),
                            _ => Ok(None),
                        }
                    }
                    Ok(Some(Err(e))) => {
                        Ok(Some(TransportEvent::Error(
                            TransportId::Connection(0),
                            e.to_string(),
                        )))
                    }
                    Ok(None) => {
                        // Stream ended
                        drop(ws_guard);
                        let _ = self.disconnect().await;
                        Ok(Some(TransportEvent::Disconnected(TransportId::Connection(0))))
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
        #[cfg(not(feature = "websocket"))]
        {
            Ok(None)
        }
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
