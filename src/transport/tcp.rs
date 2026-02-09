//! TCP transport implementation

use crate::error::{Error, Result};
use crate::transport::{TransportConfig, TransportEvent, TransportId, TransportService, TransportType};
use async_trait::async_trait;
use bytes::Bytes;
use std::collections::VecDeque;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use std::sync::Arc;

/// TCP transport implementation
pub struct TcpTransport {
    config: TransportConfig,
    stream: Arc<Mutex<Option<TcpStream>>>,
    listener: Arc<Mutex<Option<TcpListener>>>,
    events: Arc<Mutex<VecDeque<TransportEvent>>>,
    connected: Arc<Mutex<bool>>,
    next_connection_id: Arc<Mutex<u64>>,
}

impl TcpTransport {
    /// Create a new TCP transport
    pub fn new(config: TransportConfig) -> Result<Self> {
        if config.transport_type != TransportType::Tcp {
            return Err(Error::Configuration(
                "Invalid transport type for TCP".to_string(),
            ));
        }

        Ok(Self {
            config,
            stream: Arc::new(Mutex::new(None)),
            listener: Arc::new(Mutex::new(None)),
            events: Arc::new(Mutex::new(VecDeque::new())),
            connected: Arc::new(Mutex::new(false)),
            next_connection_id: Arc::new(Mutex::new(0)),
        })
    }

    async fn next_id(&self) -> u64 {
        let mut id = self.next_connection_id.lock().await;
        let current = *id;
        *id += 1;
        current
    }
}

#[async_trait]
impl TransportService for TcpTransport {
    async fn connect(&mut self) -> Result<()> {
        if self.config.is_server {
            // Server mode: bind and listen
            let listener = TcpListener::bind(&self.config.address).await?;
            let local_addr = listener.local_addr()?;
            *self.listener.lock().await = Some(listener);
            *self.connected.lock().await = true;
            
            self.events.lock().await.push_back(TransportEvent::Connected(
                TransportId::Connection(0)
            ));
            
            log::info!("TCP server listening on {}", local_addr);
            Ok(())
        } else {
            // Client mode: connect to server
            let stream = TcpStream::connect(&self.config.address).await?;
            let peer_addr = stream.peer_addr()?;
            *self.stream.lock().await = Some(stream);
            *self.connected.lock().await = true;
            
            let id = self.next_id().await;
            self.events.lock().await.push_back(TransportEvent::Connected(
                TransportId::Connection(id)
            ));
            
            log::info!("TCP client connected to {}", peer_addr);
            Ok(())
        }
    }

    async fn disconnect(&mut self) -> Result<()> {
        *self.stream.lock().await = None;
        *self.listener.lock().await = None;
        *self.connected.lock().await = false;
        
        self.events.lock().await.push_back(TransportEvent::Disconnected(
            TransportId::Connection(0)
        ));
        
        Ok(())
    }

    async fn send(&mut self, _id: &TransportId, data: Bytes) -> Result<()> {
        let mut stream_guard = self.stream.lock().await;
        if let Some(stream) = stream_guard.as_mut() {
            stream.write_all(&data).await?;
            stream.flush().await?;
            Ok(())
        } else {
            Err(Error::Connection("Not connected".to_string()))
        }
    }

    async fn poll_event(&mut self) -> Result<Option<TransportEvent>> {
        // First, check if there are any queued events
        let mut events = self.events.lock().await;
        if let Some(event) = events.pop_front() {
            return Ok(Some(event));
        }
        drop(events);

        // Try to read data if connected
        let mut stream_guard = self.stream.lock().await;
        if let Some(stream) = stream_guard.as_mut() {
            let mut buffer = vec![0u8; 4096];
            
            // Non-blocking read
            match tokio::time::timeout(
                std::time::Duration::from_millis(10),
                stream.read(&mut buffer)
            ).await {
                Ok(Ok(0)) => {
                    // Connection closed
                    drop(stream_guard);
                    let _ = self.disconnect().await;
                    Ok(Some(TransportEvent::Disconnected(TransportId::Connection(0))))
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

    fn is_connected(&self) -> bool {
        // We can't await in a non-async function, so we use try_lock
        self.connected.try_lock().map(|g| *g).unwrap_or(false)
    }

    fn transport_type(&self) -> TransportType {
        TransportType::Tcp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tcp_transport_creation() {
        let config = TransportConfig::tcp("127.0.0.1:8080").unwrap();
        let transport = TcpTransport::new(config);
        assert!(transport.is_ok());
    }

    #[tokio::test]
    async fn test_tcp_transport_wrong_type() {
        let mut config = TransportConfig::udp("127.0.0.1:8080").unwrap();
        config.transport_type = TransportType::Udp;
        let transport = TcpTransport::new(config);
        assert!(transport.is_err());
    }

    #[tokio::test]
    async fn test_tcp_server_bind() {
        let config = TransportConfig::tcp_server("127.0.0.1:0").unwrap();
        let mut transport = TcpTransport::new(config).unwrap();
        let result = transport.connect().await;
        assert!(result.is_ok());
        assert!(transport.is_connected());
    }

    #[tokio::test]
    async fn test_tcp_client_server_communication() {
        // Start server
        let server_config = TransportConfig::tcp_server("127.0.0.1:0").unwrap();
        let mut server = TcpTransport::new(server_config).unwrap();
        server.connect().await.unwrap();

        // Get the actual port the server is listening on
        let listener = server.listener.lock().await;
        let server_addr = listener.as_ref().unwrap().local_addr().unwrap();
        drop(listener);

        // Connect client
        let client_config = TransportConfig::tcp(&server_addr.to_string()).unwrap();
        let mut client = TcpTransport::new(client_config).unwrap();
        
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let result = client.connect().await;
        assert!(result.is_ok());
        assert!(client.is_connected());
    }
}
