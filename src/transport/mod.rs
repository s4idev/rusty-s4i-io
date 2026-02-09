//! Transport layer module
//!
//! This module provides unified interfaces for various transport protocols.

#[cfg(feature = "ble")]
pub mod ble;
pub mod common;
#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "serial")]
pub mod serial;
#[cfg(feature = "tcp")]
pub mod tcp;
#[cfg(feature = "tls")]
pub mod tls;
#[cfg(feature = "udp")]
pub mod udp;
#[cfg(feature = "usb")]
pub mod usb;
#[cfg(feature = "websocket")]
pub mod websocket;

pub use common::{TransportConfig, TransportEvent, TransportId, TransportService, TransportType};
