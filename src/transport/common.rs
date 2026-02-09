//! Common types and traits for transport layer

use crate::error::Result;
use async_trait::async_trait;
use bytes::Bytes;
use std::fmt;

/// Transport identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TransportId {
    /// Specific connection ID
    Connection(u64),
    /// Broadcast to all connections (for server transports)
    Broadcast,
}

/// Transport events
#[derive(Debug, Clone)]
pub enum TransportEvent {
    /// Connection established
    Connected(TransportId),
    /// Connection closed
    Disconnected(TransportId),
    /// Data received
    DataReceived(TransportId, Bytes),
    /// Error occurred
    Error(TransportId, String),
}

/// Transport type enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum TransportType {
    Tcp,
    Udp,
    Http,
    Https,
    Serial,
    UsbHid,
    Ble,
    WebSocket,
    Tls,
}

impl fmt::Display for TransportType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransportType::Tcp => write!(f, "TCP"),
            TransportType::Udp => write!(f, "UDP"),
            TransportType::Http => write!(f, "HTTP"),
            TransportType::Https => write!(f, "HTTPS"),
            TransportType::Serial => write!(f, "Serial"),
            TransportType::UsbHid => write!(f, "USB HID"),
            TransportType::Ble => write!(f, "BLE"),
            TransportType::WebSocket => write!(f, "WebSocket"),
            TransportType::Tls => write!(f, "TLS"),
        }
    }
}

/// Transport configuration
#[derive(Debug, Clone)]
pub struct TransportConfig {
    pub transport_type: TransportType,
    pub address: String,
    pub is_server: bool,
    pub options: TransportOptions,
}

/// Transport-specific options
#[derive(Debug, Clone, Default)]
pub struct TransportOptions {
    pub timeout_ms: Option<u64>,
    pub buffer_size: Option<usize>,
    pub retry_count: Option<u32>,
    // Serial-specific
    pub baud_rate: Option<u32>,
    pub data_bits: Option<u8>,
    // USB-specific
    pub vendor_id: Option<u16>,
    pub product_id: Option<u16>,
    // BLE-specific
    pub service_uuid: Option<String>,
}

impl TransportConfig {
    /// Create TCP configuration
    pub fn tcp(address: &str) -> Result<Self> {
        Ok(Self {
            transport_type: TransportType::Tcp,
            address: address.to_string(),
            is_server: false,
            options: TransportOptions::default(),
        })
    }

    /// Create TCP server configuration
    pub fn tcp_server(address: &str) -> Result<Self> {
        Ok(Self {
            transport_type: TransportType::Tcp,
            address: address.to_string(),
            is_server: true,
            options: TransportOptions::default(),
        })
    }

    /// Create UDP configuration
    pub fn udp(address: &str) -> Result<Self> {
        Ok(Self {
            transport_type: TransportType::Udp,
            address: address.to_string(),
            is_server: false,
            options: TransportOptions::default(),
        })
    }

    /// Parse from URL-like string (e.g., "tcp://localhost:8080")
    pub fn from_url(url: &str) -> Result<Self> {
        let parts: Vec<&str> = url.split("://").collect();
        if parts.len() != 2 {
            return Err(crate::error::Error::Configuration(
                "Invalid URL format".to_string(),
            ));
        }

        let transport_type = match parts[0].to_lowercase().as_str() {
            "tcp" => TransportType::Tcp,
            "udp" => TransportType::Udp,
            "http" => TransportType::Http,
            "https" => TransportType::Https,
            "ws" => TransportType::WebSocket,
            "wss" => TransportType::WebSocket,
            "serial" => TransportType::Serial,
            "usb" => TransportType::UsbHid,
            "ble" => TransportType::Ble,
            "tls" => TransportType::Tls,
            _ => {
                return Err(crate::error::Error::Configuration(format!(
                    "Unknown transport type: {}",
                    parts[0]
                )))
            }
        };

        Ok(Self {
            transport_type,
            address: parts[1].to_string(),
            is_server: false,
            options: TransportOptions::default(),
        })
    }
}

/// Transport service trait
#[async_trait]
pub trait TransportService: Send + Sync {
    /// Connect to the transport
    async fn connect(&mut self) -> Result<()>;

    /// Disconnect from the transport
    async fn disconnect(&mut self) -> Result<()>;

    /// Send data through the transport
    async fn send(&mut self, id: &TransportId, data: Bytes) -> Result<()>;

    /// Receive events (non-blocking)
    async fn poll_event(&mut self) -> Result<Option<TransportEvent>>;

    /// Check if connected
    fn is_connected(&self) -> bool;

    /// Get transport type
    fn transport_type(&self) -> TransportType;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_config_tcp() {
        let config = TransportConfig::tcp("127.0.0.1:8080").unwrap();
        assert_eq!(config.transport_type, TransportType::Tcp);
        assert_eq!(config.address, "127.0.0.1:8080");
        assert!(!config.is_server);
    }

    #[test]
    fn test_transport_config_from_url() {
        let config = TransportConfig::from_url("tcp://localhost:9000").unwrap();
        assert_eq!(config.transport_type, TransportType::Tcp);
        assert_eq!(config.address, "localhost:9000");
    }

    #[test]
    fn test_transport_id_equality() {
        let id1 = TransportId::Connection(1);
        let id2 = TransportId::Connection(1);
        let id3 = TransportId::Connection(2);
        
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_transport_type_display() {
        assert_eq!(TransportType::Tcp.to_string(), "TCP");
        assert_eq!(TransportType::WebSocket.to_string(), "WebSocket");
    }
}
