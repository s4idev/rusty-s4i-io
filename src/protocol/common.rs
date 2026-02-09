//! Common protocol types and traits

use crate::error::Result;
use async_trait::async_trait;
use bytes::Bytes;

/// Protocol message
#[derive(Debug, Clone)]
pub struct ProtocolMessage {
    pub topic: String,
    pub payload: Bytes,
    pub qos: u8,
}

/// Protocol handler trait
#[async_trait]
pub trait ProtocolHandler: Send + Sync {
    /// Connect to the protocol endpoint
    async fn connect(&mut self) -> Result<()>;

    /// Disconnect from the protocol endpoint
    async fn disconnect(&mut self) -> Result<()>;

    /// Publish a message
    async fn publish(&mut self, message: ProtocolMessage) -> Result<()>;

    /// Subscribe to a topic
    async fn subscribe(&mut self, topic: &str) -> Result<()>;

    /// Poll for incoming messages
    async fn poll_message(&mut self) -> Result<Option<ProtocolMessage>>;

    /// Check if connected
    fn is_connected(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_message_creation() {
        let msg = ProtocolMessage {
            topic: "test/topic".to_string(),
            payload: Bytes::from("test data"),
            qos: 1,
        };
        assert_eq!(msg.topic, "test/topic");
        assert_eq!(msg.qos, 1);
    }
}
