//! MQTT protocol implementation

use crate::error::{Error, Result};
use crate::protocol::{ProtocolHandler, ProtocolMessage};
use async_trait::async_trait;
use bytes::Bytes;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(feature = "mqtt")]
use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet, QoS};

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
    #[cfg(feature = "mqtt")]
    client: Arc<Mutex<Option<AsyncClient>>>,
    #[cfg(feature = "mqtt")]
    eventloop: Arc<Mutex<Option<EventLoop>>>,
    connected: Arc<Mutex<bool>>,
    messages: Arc<Mutex<VecDeque<ProtocolMessage>>>,
}

impl MqttHandler {
    /// Create a new MQTT handler
    pub fn new(config: MqttConfig) -> Self {
        Self {
            config,
            #[cfg(feature = "mqtt")]
            client: Arc::new(Mutex::new(None)),
            #[cfg(feature = "mqtt")]
            eventloop: Arc::new(Mutex::new(None)),
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
            // Parse broker URL
            let url = self.config.broker_url.trim_start_matches("mqtt://");
            let parts: Vec<&str> = url.split(':').collect();
            let host = parts.get(0).unwrap_or(&"localhost").to_string();
            let port: u16 = parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(1883);

            // Create MQTT options
            let mut mqttoptions = MqttOptions::new(&self.config.client_id, host, port);
            mqttoptions.set_keep_alive(std::time::Duration::from_secs(self.config.keep_alive));
            mqttoptions.set_clean_session(self.config.clean_session);

            if let (Some(username), Some(password)) = (&self.config.username, &self.config.password) {
                mqttoptions.set_credentials(username, password);
            }

            // Create async client and event loop
            let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

            // Poll once to establish connection
            match tokio::time::timeout(
                std::time::Duration::from_secs(5),
                eventloop.poll()
            ).await {
                Ok(Ok(Event::Incoming(Packet::ConnAck(_)))) => {
                    log::info!("MQTT connected to {}", self.config.broker_url);
                }
                Ok(Ok(_)) => {
                    // Connection in progress
                }
                Ok(Err(e)) => {
                    return Err(Error::Connection(format!("MQTT connection failed: {}", e)));
                }
                Err(_) => {
                    return Err(Error::Timeout);
                }
            }

            *self.client.lock().await = Some(client);
            *self.eventloop.lock().await = Some(eventloop);
            *self.connected.lock().await = true;
            
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
            let mut client_guard = self.client.lock().await;
            if let Some(client) = client_guard.take() {
                let _ = client.disconnect().await;
            }
            *self.eventloop.lock().await = None;
        }
        
        *self.connected.lock().await = false;
        log::info!("MQTT disconnected");
        Ok(())
    }

    async fn publish(&mut self, message: ProtocolMessage) -> Result<()> {
        #[cfg(feature = "mqtt")]
        {
            let client_guard = self.client.lock().await;
            if let Some(client) = client_guard.as_ref() {
                let qos = match message.qos {
                    0 => QoS::AtMostOnce,
                    1 => QoS::AtLeastOnce,
                    2 => QoS::ExactlyOnce,
                    _ => QoS::AtMostOnce,
                };

                client
                    .publish(&message.topic, qos, false, message.payload.to_vec())
                    .await
                    .map_err(|e| Error::Protocol(format!("MQTT publish failed: {}", e)))?;

                log::debug!("Published to topic: {}", message.topic);
                Ok(())
            } else {
                Err(Error::Connection("Not connected to MQTT broker".to_string()))
            }
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
            let client_guard = self.client.lock().await;
            if let Some(client) = client_guard.as_ref() {
                client
                    .subscribe(topic, QoS::AtMostOnce)
                    .await
                    .map_err(|e| Error::Protocol(format!("MQTT subscribe failed: {}", e)))?;

                log::debug!("Subscribed to topic: {}", topic);
                Ok(())
            } else {
                Err(Error::Connection("Not connected to MQTT broker".to_string()))
            }
        }
        #[cfg(not(feature = "mqtt"))]
        {
            let _ = topic;
            Err(Error::NotSupported("MQTT feature not enabled".to_string()))
        }
    }

    async fn poll_message(&mut self) -> Result<Option<ProtocolMessage>> {
        // First check queued messages
        let mut messages = self.messages.lock().await;
        if let Some(msg) = messages.pop_front() {
            return Ok(Some(msg));
        }
        drop(messages);

        #[cfg(feature = "mqtt")]
        {
            // Poll event loop for new messages
            let mut eventloop_guard = self.eventloop.lock().await;
            if let Some(eventloop) = eventloop_guard.as_mut() {
                match tokio::time::timeout(
                    std::time::Duration::from_millis(10),
                    eventloop.poll()
                ).await {
                    Ok(Ok(Event::Incoming(Packet::Publish(publish)))) => {
                        let msg = ProtocolMessage {
                            topic: publish.topic.clone(),
                            payload: Bytes::from(publish.payload.to_vec()),
                            qos: publish.qos as u8,
                        };
                        Ok(Some(msg))
                    }
                    Ok(Ok(_)) => Ok(None),
                    Ok(Err(_)) => Ok(None),
                    Err(_) => Ok(None), // Timeout
                }
            } else {
                Ok(None)
            }
        }
        #[cfg(not(feature = "mqtt"))]
        {
            Ok(None)
        }
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
