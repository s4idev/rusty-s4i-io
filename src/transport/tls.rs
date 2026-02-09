//! TLS/SSL transport implementation

use crate::error::{Error, Result};
use crate::transport::{
    TransportConfig, TransportEvent, TransportId, TransportService, TransportType,
};
use async_trait::async_trait;
use bytes::Bytes;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(feature = "tls")]
use tokio::io::{AsyncReadExt, AsyncWriteExt};
#[cfg(feature = "tls")]
use tokio::net::TcpStream;
#[cfg(feature = "tls")]
use tokio_native_tls::{TlsConnector, TlsStream};

/// TLS transport implementation
pub struct TlsTransport {
    config: TransportConfig,
    #[cfg(feature = "tls")]
    stream: Arc<Mutex<Option<TlsStream<TcpStream>>>>,
    events: Arc<Mutex<VecDeque<TransportEvent>>>,
    connected: Arc<Mutex<bool>>,
}

impl TlsTransport {
    /// Create a new TLS transport
    pub fn new(config: TransportConfig) -> Result<Self> {
        if config.transport_type != TransportType::Tls {
            return Err(Error::Configuration(
                "Invalid transport type for TLS".to_string(),
            ));
        }

        Ok(Self {
            config,
            #[cfg(feature = "tls")]
            stream: Arc::new(Mutex::new(None)),
            events: Arc::new(Mutex::new(VecDeque::new())),
            connected: Arc::new(Mutex::new(false)),
        })
    }
}

#[async_trait]
impl TransportService for TlsTransport {
    async fn connect(&mut self) -> Result<()> {
        #[cfg(feature = "tls")]
        {
            // Parse host:port
            let parts: Vec<&str> = self.config.address.split(':').collect();
            if parts.len() != 2 {
                return Err(Error::Configuration("Invalid address format, expected host:port".to_string()));
            }
            let host = parts[0];

            // Create TCP connection
            let tcp_stream = TcpStream::connect(&self.config.address).await?;

            // Create TLS connector
            let cx = native_tls::TlsConnector::builder()
                .build()
                .map_err(|e| Error::Connection(format!("TLS connector build failed: {}", e)))?;
            let cx = TlsConnector::from(cx);

            // Perform TLS handshake
            let tls_stream = cx
                .connect(host, tcp_stream)
                .await
                .map_err(|e| Error::Connection(format!("TLS handshake failed: {}", e)))?;

            *self.stream.lock().await = Some(tls_stream);
            *self.connected.lock().await = true;
            
            self.events
                .lock()
                .await
                .push_back(TransportEvent::Connected(TransportId::Connection(0)));
            
            log::info!("TLS connected to {}", self.config.address);
            Ok(())
        }
        #[cfg(not(feature = "tls"))]
        {
            Err(Error::NotSupported("TLS feature not enabled".to_string()))
        }
    }

    async fn disconnect(&mut self) -> Result<()> {
        #[cfg(feature = "tls")]
        {
            *self.stream.lock().await = None;
        }
        
        *self.connected.lock().await = false;
        self.events
            .lock()
            .await
            .push_back(TransportEvent::Disconnected(TransportId::Connection(0)));
        
        log::info!("TLS disconnected");
        Ok(())
    }

    async fn send(&mut self, _id: &TransportId, data: Bytes) -> Result<()> {
        #[cfg(feature = "tls")]
        {
            let mut stream_guard = self.stream.lock().await;
            if let Some(stream) = stream_guard.as_mut() {
                stream.write_all(&data).await?;
                stream.flush().await?;
                Ok(())
            } else {
                Err(Error::Connection("Not connected".to_string()))
            }
        }
        #[cfg(not(feature = "tls"))]
        {
            let _ = data;
            Err(Error::NotSupported("TLS feature not enabled".to_string()))
        }
    }

    async fn poll_event(&mut self) -> Result<Option<TransportEvent>> {
        // First, check if there are any queued events
        let mut events = self.events.lock().await;
        if let Some(event) = events.pop_front() {
            return Ok(Some(event));
        }
        drop(events);

        #[cfg(feature = "tls")]
        {
            // Try to read data from TLS stream
            let mut stream_guard = self.stream.lock().await;
            if let Some(stream) = stream_guard.as_mut() {
                let mut buffer = vec![0u8; 4096];

                // Non-blocking read with timeout
                match tokio::time::timeout(
                    std::time::Duration::from_millis(10),
                    stream.read(&mut buffer),
                )
                .await
                {
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
        #[cfg(not(feature = "tls"))]
        {
            Ok(None)
        }
    }

    fn is_connected(&self) -> bool {
        self.connected.try_lock().map(|g| *g).unwrap_or(false)
    }

    fn transport_type(&self) -> TransportType {
        TransportType::Tls
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tls_transport_creation() {
        let config = TransportConfig {
            transport_type: TransportType::Tls,
            address: "localhost:443".to_string(),
            is_server: false,
            options: Default::default(),
        };
        let transport = TlsTransport::new(config);
        assert!(transport.is_ok());
    }
}
