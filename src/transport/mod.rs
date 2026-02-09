//! Transport layer module
//!
//! This module provides unified interfaces for various transport protocols.

pub mod common;
#[cfg(feature = "tcp")]
pub mod tcp;
#[cfg(feature = "udp")]
pub mod udp;
#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "serial")]
pub mod serial;
#[cfg(feature = "usb")]
pub mod usb;
#[cfg(feature = "ble")]
pub mod ble;
#[cfg(feature = "websocket")]
pub mod websocket;
#[cfg(feature = "tls")]
pub mod tls;

pub use common::{TransportService, TransportEvent, TransportId, TransportConfig, TransportType};
