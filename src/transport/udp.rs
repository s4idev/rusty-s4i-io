//! UDP transport implementation

use crate::error::{Error, Result};
use crate::transport::{TransportConfig, TransportEvent, TransportId, TransportService, TransportType};
use async_trait::async_trait;
use bytes::Bytes;
use std::collections::VecDeque;
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use std::sync::Arc;

/// UDP transport implementation
pub struct UdpTransport {
    config: TransportConfig,
    socket: Arc<Mutex<Option<UdpSocket>>>,
    events: Arc<Mutex<VecDeque<TransportEvent>>>,
    connected: Arc<Mutex<bool>>,
    remote_addr: Arc<Mutex<Option<SocketAddr>>>,
}

impl UdpTransport {
    /// Create a new UDP transport
    pub fn new(config: TransportConfig) -> Result<Self> {
        if config.transport_type != TransportType::Udp {
            return Err(Error::Configuration(
                "Invalid transport type for UDP".to_string(),
            ));
        }

        Ok(Self {
            config,
            socket: Arc::new(Mutex::new(None)),
            events: Arc::new(Mutex::new(VecDeque::new())),
            connected: Arc::new(Mutex::new(false)),
            remote_addr: Arc::new(Mutex::new(None)),
        })
    }
}

#[async_trait]
impl TransportService for UdpTransport {
    async fn connect(&mut self) -> Result<()> {
        if self.config.is_server {
            // Server mode: bind to local address
            let socket = UdpSocket::bind(&self.config.address).await?;
            let local_addr = socket.local_addr()?;
            *self.socket.lock().await = Some(socket);
            *self.connected.lock().await = true;
            
            self.events.lock().await.push_back(TransportEvent::Connected(
                TransportId::Connection(0)
            ));
            
            log::info!("UDP socket bound to {}", local_addr);
            Ok(())
        } else {
            // Client mode: bind to any local port and set remote address
            let socket = UdpSocket::bind("0.0.0.0:0").await?;
            let remote: SocketAddr = self.config.address.parse()
                .map_err(|e| Error::Configuration(format!("Invalid address: {}", e)))?;
            
            socket.connect(&remote).await?;
            *self.remote_addr.lock().await = Some(remote);
            *self.socket.lock().await = Some(socket);
            *self.connected.lock().await = true;
            
            self.events.lock().await.push_back(TransportEvent::Connected(
                TransportId::Connection(0)
            ));
            
            log::info!("UDP socket connected to {}", remote);
            Ok(())
        }
    }

    async fn disconnect(&mut self) -> Result<()> {
        *self.socket.lock().await = None;
        *self.connected.lock().await = false;
        *self.remote_addr.lock().await = None;
        
        self.events.lock().await.push_back(TransportEvent::Disconnected(
            TransportId::Connection(0)
        ));
        
        Ok(())
    }

    async fn send(&mut self, id: &TransportId, data: Bytes) -> Result<()> {
        let socket_guard = self.socket.lock().await;
        if let Some(socket) = socket_guard.as_ref() {
            match id {
                TransportId::Broadcast => {
                    // Broadcast mode
                    socket.set_broadcast(true)?;
                    socket.send(&data).await?;
                }
                TransportId::Connection(_) => {
                    socket.send(&data).await?;
                }
            }
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
        let socket_guard = self.socket.lock().await;
        if let Some(socket) = socket_guard.as_ref() {
            let mut buffer = vec![0u8; 65536]; // Max UDP packet size
            
            // Non-blocking read
            match tokio::time::timeout(
                std::time::Duration::from_millis(10),
                socket.recv_from(&mut buffer)
            ).await {
                Ok(Ok((n, _addr))) => {
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
        self.connected.try_lock().map(|g| *g).unwrap_or(false)
    }

    fn transport_type(&self) -> TransportType {
        TransportType::Udp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_udp_transport_creation() {
        let config = TransportConfig::udp("127.0.0.1:8080").unwrap();
        let transport = UdpTransport::new(config);
        assert!(transport.is_ok());
    }

    #[tokio::test]
    async fn test_udp_server_bind() {
        let config = TransportConfig {
            transport_type: TransportType::Udp,
            address: "127.0.0.1:0".to_string(),
            is_server: true,
            options: Default::default(),
        };
        let mut transport = UdpTransport::new(config).unwrap();
        let result = transport.connect().await;
        assert!(result.is_ok());
        assert!(transport.is_connected());
    }

    #[tokio::test]
    async fn test_udp_client_connect() {
        let config = TransportConfig::udp("127.0.0.1:9999").unwrap();
        let mut transport = UdpTransport::new(config).unwrap();
        let result = transport.connect().await;
        assert!(result.is_ok());
        assert!(transport.is_connected());
    }

    #[tokio::test]
    async fn test_udp_send_receive() {
        // Create server
        let server_config = TransportConfig {
            transport_type: TransportType::Udp,
            address: "127.0.0.1:0".to_string(),
            is_server: true,
            options: Default::default(),
        };
        let mut server = UdpTransport::new(server_config).unwrap();
        server.connect().await.unwrap();

        // Get server address
        let socket = server.socket.lock().await;
        let server_addr = socket.as_ref().unwrap().local_addr().unwrap();
        drop(socket);

        // Create client
        let client_config = TransportConfig::udp(&server_addr.to_string()).unwrap();
        let mut client = UdpTransport::new(client_config).unwrap();
        client.connect().await.unwrap();

        // Send data from client
        let test_data = Bytes::from("Hello UDP");
        client.send(&TransportId::Connection(0), test_data.clone()).await.unwrap();

        // Receive on server - first consume the Connected event
        let event = server.poll_event().await.unwrap();
        assert!(matches!(event, Some(TransportEvent::Connected(_))));

        // Now receive the data
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let event = server.poll_event().await.unwrap();
        
        match event {
            Some(TransportEvent::DataReceived(_, data)) => {
                assert_eq!(data, test_data);
            }
            other => panic!("Expected DataReceived event, got {:?}", other),
        }
    }
}
