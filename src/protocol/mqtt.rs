//! MQTT protocol implementation

use crate::error::{Error, Result};
use crate::protocol::{ProtocolHandler, ProtocolMessage};
use async_trait::async_trait;
use std::collections::VecDeque;
use tokio::sync::Mutex;
use std::sync::Arc;

/// MQTT client configuration
#[derive(Debug, Clone)]
pub struct MqttConfig {
    pub broker_url: String,
    pub client_id: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub clean_session: bool,
    pub keep_alive: u64,
}

impl Default for MqttConfig {
    fn default() -> Self {
        Self {
            broker_url: "mqtt://localhost:1883".to_string(),
            client_id: "rusty-s4i-io-client".to_string(),
            username: None,
            password: None,
            clean_session: true,
            keep_alive: 60,
        }
    }
}

/// MQTT protocol handler
pub struct MqttHandler {
    config: MqttConfig,
    connected: Arc<Mutex<bool>>,
    messages: Arc<Mutex<VecDeque<ProtocolMessage>>>,
}

impl MqttHandler {
    /// Create a new MQTT handler
    pub fn new(config: MqttConfig) -> Self {
        Self {
            config,
            connected: Arc::new(Mutex::new(false)),
            messages: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
}

#[async_trait]
impl ProtocolHandler for MqttHandler {
    async fn connect(&mut self) -> Result<()> {
        #[cfg(feature = "mqtt")]
        {
            // MQTT connection implementation using rumqttc would go here
            *self.connected.lock().await = true;
            log::info!("MQTT connected to {}", self.config.broker_url);
            Ok(())
        }
        #[cfg(not(feature = "mqtt"))]
        {
            Err(Error::NotSupported("MQTT feature not enabled".to_string()))
        }
    }

    async fn disconnect(&mut self) -> Result<()> {
        #[cfg(feature = "mqtt")]
        {
            *self.connected.lock().await = false;
            log::info!("MQTT disconnected");
            Ok(())
        }
        #[cfg(not(feature = "mqtt"))]
        {
            Err(Error::NotSupported("MQTT feature not enabled".to_string()))
        }
    }

    async fn publish(&mut self, message: ProtocolMessage) -> Result<()> {
        #[cfg(feature = "mqtt")]
        {
            if !*self.connected.lock().await {
                return Err(Error::Connection("Not connected to MQTT broker".to_string()));
            }
            // Publish implementation would go here
            log::debug!("Publishing to topic: {}", message.topic);
            Ok(())
        }
        #[cfg(not(feature = "mqtt"))]
        {
            let _ = message;
            Err(Error::NotSupported("MQTT feature not enabled".to_string()))
        }
    }

    async fn subscribe(&mut self, topic: &str) -> Result<()> {
        #[cfg(feature = "mqtt")]
        {
            if !*self.connected.lock().await {
                return Err(Error::Connection("Not connected to MQTT broker".to_string()));
            }
            // Subscribe implementation would go here
            log::debug!("Subscribing to topic: {}", topic);
            Ok(())
        }
        #[cfg(not(feature = "mqtt"))]
        {
            let _ = topic;
            Err(Error::NotSupported("MQTT feature not enabled".to_string()))
        }
    }

    async fn poll_message(&mut self) -> Result<Option<ProtocolMessage>> {
        let mut messages = self.messages.lock().await;
        Ok(messages.pop_front())
    }

    fn is_connected(&self) -> bool {
        self.connected.try_lock().map(|g| *g).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mqtt_config_default() {
        let config = MqttConfig::default();
        assert_eq!(config.broker_url, "mqtt://localhost:1883");
        assert_eq!(config.client_id, "rusty-s4i-io-client");
        assert!(config.clean_session);
    }

    #[test]
    fn test_mqtt_handler_creation() {
        let config = MqttConfig::default();
        let handler = MqttHandler::new(config);
        assert!(!handler.is_connected());
    }
}
