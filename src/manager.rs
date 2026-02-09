//! Transport Manager
//!
//! Manages multiple transport services

use crate::error::{Error, Result};
use crate::transport::{TransportConfig, TransportEvent, TransportId, TransportService};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Transport manager for managing multiple transports
pub struct TransportManager {
    transports: Arc<Mutex<HashMap<String, Box<dyn TransportService>>>>,
}

impl TransportManager {
    /// Create a new transport manager
    pub fn new() -> Self {
        Self {
            transports: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Add a transport to the manager
    pub async fn add_transport(&mut self, config: TransportConfig) -> Result<String> {
        let transport_id = format!("{}_{}", config.transport_type, config.address);

        let transport: Box<dyn TransportService> = match config.transport_type {
            #[cfg(feature = "tcp")]
            crate::transport::TransportType::Tcp => {
                Box::new(crate::transport::tcp::TcpTransport::new(config)?)
            }
            #[cfg(feature = "udp")]
            crate::transport::TransportType::Udp => {
                Box::new(crate::transport::udp::UdpTransport::new(config)?)
            }
            #[cfg(feature = "http")]
            crate::transport::TransportType::Http | crate::transport::TransportType::Https => {
                Box::new(crate::transport::http::HttpTransport::new(config)?)
            }
            #[cfg(feature = "websocket")]
            crate::transport::TransportType::WebSocket => Box::new(
                crate::transport::websocket::WebSocketTransport::new(config)?,
            ),
            #[cfg(feature = "serial")]
            crate::transport::TransportType::Serial => {
                Box::new(crate::transport::serial::SerialTransport::new(config)?)
            }
            #[cfg(feature = "usb")]
            crate::transport::TransportType::UsbHid => {
                Box::new(crate::transport::usb::UsbTransport::new(config)?)
            }
            #[cfg(feature = "ble")]
            crate::transport::TransportType::Ble => {
                Box::new(crate::transport::ble::BleTransport::new(config)?)
            }
            #[cfg(feature = "tls")]
            crate::transport::TransportType::Tls => {
                Box::new(crate::transport::tls::TlsTransport::new(config)?)
            }
            #[cfg(not(feature = "tcp"))]
            crate::transport::TransportType::Tcp => {
                return Err(Error::NotSupported(
                    "TCP transport not enabled. Enable the 'tcp' feature flag.".to_string(),
                ))
            }
            #[cfg(not(feature = "udp"))]
            crate::transport::TransportType::Udp => {
                return Err(Error::NotSupported(
                    "UDP transport not enabled. Enable the 'udp' feature flag.".to_string(),
                ))
            }
            #[cfg(not(feature = "http"))]
            crate::transport::TransportType::Http | crate::transport::TransportType::Https => {
                return Err(Error::NotSupported(
                    "HTTP transport not enabled. Enable the 'http' feature flag.".to_string(),
                ))
            }
            #[cfg(not(feature = "websocket"))]
            crate::transport::TransportType::WebSocket => {
                return Err(Error::NotSupported(
                    "WebSocket transport not enabled. Enable the 'websocket' feature flag."
                        .to_string(),
                ))
            }
            #[cfg(not(feature = "serial"))]
            crate::transport::TransportType::Serial => {
                return Err(Error::NotSupported(
                    "Serial transport not enabled. Enable the 'serial' feature flag.".to_string(),
                ))
            }
            #[cfg(not(feature = "usb"))]
            crate::transport::TransportType::UsbHid => {
                return Err(Error::NotSupported(
                    "USB HID transport not enabled. Enable the 'usb' feature flag.".to_string(),
                ))
            }
            #[cfg(not(feature = "ble"))]
            crate::transport::TransportType::Ble => {
                return Err(Error::NotSupported(
                    "BLE transport not enabled. Enable the 'ble' feature flag.".to_string(),
                ))
            }
            #[cfg(not(feature = "tls"))]
            crate::transport::TransportType::Tls => {
                return Err(Error::NotSupported(
                    "TLS transport not enabled. Enable the 'tls' feature flag.".to_string(),
                ))
            }
        };

        let mut transports = self.transports.lock().await;
        transports.insert(transport_id.clone(), transport);
        Ok(transport_id)
    }

    /// Connect a specific transport
    pub async fn connect(&self, transport_id: &str) -> Result<()> {
        let mut transports = self.transports.lock().await;
        if let Some(transport) = transports.get_mut(transport_id) {
            transport.connect().await
        } else {
            Err(Error::Configuration(format!(
                "Transport not found: {}",
                transport_id
            )))
        }
    }

    /// Disconnect a specific transport
    pub async fn disconnect(&self, transport_id: &str) -> Result<()> {
        let mut transports = self.transports.lock().await;
        if let Some(transport) = transports.get_mut(transport_id) {
            transport.disconnect().await
        } else {
            Err(Error::Configuration(format!(
                "Transport not found: {}",
                transport_id
            )))
        }
    }

    /// Send data through a specific transport
    pub async fn send(
        &self,
        transport_id: &str,
        id: &TransportId,
        data: bytes::Bytes,
    ) -> Result<()> {
        let mut transports = self.transports.lock().await;
        if let Some(transport) = transports.get_mut(transport_id) {
            transport.send(id, data).await
        } else {
            Err(Error::Configuration(format!(
                "Transport not found: {}",
                transport_id
            )))
        }
    }

    /// Poll for events from all transports
    pub async fn poll_events(&self) -> Result<Vec<(String, TransportEvent)>> {
        let mut transports = self.transports.lock().await;
        let mut events = Vec::new();

        for (id, transport) in transports.iter_mut() {
            if let Some(event) = transport.poll_event().await? {
                events.push((id.clone(), event));
            }
        }

        Ok(events)
    }

    /// Remove a transport
    pub async fn remove_transport(&mut self, transport_id: &str) -> Result<()> {
        let mut transports = self.transports.lock().await;
        if transports.remove(transport_id).is_some() {
            Ok(())
        } else {
            Err(Error::Configuration(format!(
                "Transport not found: {}",
                transport_id
            )))
        }
    }
}

impl Default for TransportManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_transport_manager_new() {
        let manager = TransportManager::new();
        let transports = manager.transports.lock().await;
        assert_eq!(transports.len(), 0);
    }

    #[cfg(feature = "tcp")]
    #[tokio::test]
    async fn test_add_tcp_transport() {
        let mut manager = TransportManager::new();
        let config = TransportConfig::tcp("127.0.0.1:8080").unwrap();
        let result = manager.add_transport(config).await;
        assert!(result.is_ok());
    }
}
