//! # rusty-s4i-io
//!
//! A comprehensive Rust library for managing multiple transport protocols with a unified framework.
//!
//! This library provides support for various transport layers (TCP/IP, UDP, HTTP/HTTPS, Serial Port,
//! USB HID, BLE, WebSocket, SSL/TLS) and application layer protocols (MQTT, Modbus, BACNet/IP).
//!
//! ## Features
//!
//! - Multi-platform support (macOS, Linux, Windows)
//! - Unified transport interface
//! - Async/await support via Tokio
//! - Application layer protocol support
//!
//! ## Example
//!
//! ```rust,no_run
//! use rusty_s4i_io::{TransportManager, TransportConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = TransportConfig::tcp("127.0.0.1:8080")?;
//!     let mut manager = TransportManager::new();
//!     manager.add_transport(config).await?;
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod transport;
pub mod protocol;
mod manager;

pub use error::{Error, Result};
pub use transport::{TransportService, TransportEvent, TransportId, TransportConfig};
pub use manager::TransportManager;

#[cfg(test)]
mod tests {
    #[test]
    fn test_library_version() {
        // Basic test to ensure library compiles
        assert_eq!(env!("CARGO_PKG_VERSION"), "0.1.0");
    }
}
