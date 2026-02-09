//! BACNet/IP protocol implementation

use crate::error::{Error, Result};
use crate::protocol::{ProtocolHandler, ProtocolMessage};
use async_trait::async_trait;
use bytes::{Bytes, BytesMut, BufMut};
use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(feature = "bacnet")]
use tokio::net::UdpSocket;

/// BACNet object types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BacnetObjectType {
    AnalogInput = 0,
    AnalogOutput = 1,
    AnalogValue = 2,
    BinaryInput = 3,
    BinaryOutput = 4,
    BinaryValue = 5,
    Device = 8,
}

/// BACNet property identifiers
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BacnetProperty {
    PresentValue = 85,
    ObjectName = 77,
    Description = 28,
    Units = 117,
}

/// BACNet client configuration
#[derive(Debug, Clone)]
pub struct BacnetConfig {
    pub address: String,
    pub port: u16,
    pub device_id: u32,
    pub network_number: u16,
}

impl Default for BacnetConfig {
    fn default() -> Self {
        Self {
            address: "127.0.0.1".to_string(),
            port: 47808, // Standard BACNet/IP port
            device_id: 1234,
            network_number: 0,
        }
    }
}

/// BACNet protocol handler
pub struct BacnetHandler {
    config: BacnetConfig,
    #[cfg(feature = "bacnet")]
    socket: Arc<Mutex<Option<UdpSocket>>>,
    #[cfg(feature = "bacnet")]
    broadcast_addr: Arc<Mutex<Option<SocketAddr>>>,
    connected: Arc<Mutex<bool>>,
    messages: Arc<Mutex<VecDeque<ProtocolMessage>>>,
}

impl BacnetHandler {
    /// Create a new BACNet handler
    pub fn new(config: BacnetConfig) -> Self {
        Self {
            config,
            #[cfg(feature = "bacnet")]
            socket: Arc::new(Mutex::new(None)),
            #[cfg(feature = "bacnet")]
            broadcast_addr: Arc::new(Mutex::new(None)),
            connected: Arc::new(Mutex::new(false)),
            messages: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Read property
    pub async fn read_property(
        &mut self,
        device_id: u32,
        object_type: BacnetObjectType,
        object_instance: u32,
        property: BacnetProperty,
    ) -> Result<Bytes> {
        #[cfg(feature = "bacnet")]
        {
            if !*self.connected.lock().await {
                return Err(Error::Connection(
                    "Not connected to BACNet network".to_string(),
                ));
            }

            // Build BACNet Read Property request
            let request = self.build_read_property_request(
                device_id,
                object_type,
                object_instance,
                property,
            )?;

            // Send request
            let socket_guard = self.socket.lock().await;
            if let Some(socket) = socket_guard.as_ref() {
                let target = format!("{}:{}", self.config.address, self.config.port);
                socket.send_to(&request, &target).await?;

                // Wait for response (simplified - in real implementation would parse BACNet response)
                let mut buffer = vec![0u8; 1024];
                match tokio::time::timeout(
                    std::time::Duration::from_secs(5),
                    socket.recv_from(&mut buffer)
                ).await {
                    Ok(Ok((size, _))) => {
                        log::debug!(
                            "Read property {:?} from device {} object {}:{}",
                            property,
                            device_id,
                            object_type as u32,
                            object_instance
                        );
                        Ok(Bytes::copy_from_slice(&buffer[..size]))
                    }
                    Ok(Err(e)) => Err(Error::Connection(e.to_string())),
                    Err(_) => Err(Error::Timeout),
                }
            } else {
                Err(Error::Connection("Socket not available".to_string()))
            }
        }
        #[cfg(not(feature = "bacnet"))]
        {
            let _ = (device_id, object_type, object_instance, property);
            Err(Error::NotSupported(
                "BACNet feature not enabled".to_string(),
            ))
        }
    }

    /// Write property
    pub async fn write_property(
        &mut self,
        device_id: u32,
        object_type: BacnetObjectType,
        object_instance: u32,
        property: BacnetProperty,
        value: Bytes,
    ) -> Result<()> {
        #[cfg(feature = "bacnet")]
        {
            if !*self.connected.lock().await {
                return Err(Error::Connection(
                    "Not connected to BACNet network".to_string(),
                ));
            }

            // Build BACNet Write Property request
            let request = self.build_write_property_request(
                device_id,
                object_type,
                object_instance,
                property,
                value,
            )?;

            // Send request
            let socket_guard = self.socket.lock().await;
            if let Some(socket) = socket_guard.as_ref() {
                let target = format!("{}:{}", self.config.address, self.config.port);
                socket.send_to(&request, &target).await?;

                log::debug!(
                    "Wrote property {:?} to device {} object {}:{}",
                    property,
                    device_id,
                    object_type as u32,
                    object_instance
                );
                Ok(())
            } else {
                Err(Error::Connection("Socket not available".to_string()))
            }
        }
        #[cfg(not(feature = "bacnet"))]
        {
            let _ = (device_id, object_type, object_instance, property, value);
            Err(Error::NotSupported(
                "BACNet feature not enabled".to_string(),
            ))
        }
    }

    /// Perform Who-Is discovery
    pub async fn who_is(&mut self) -> Result<Vec<u32>> {
        #[cfg(feature = "bacnet")]
        {
            if !*self.connected.lock().await {
                return Err(Error::Connection(
                    "Not connected to BACNet network".to_string(),
                ));
            }

            // Build BACNet Who-Is broadcast request
            let request = self.build_who_is_request()?;

            // Send broadcast
            let socket_guard = self.socket.lock().await;
            let broadcast_guard = self.broadcast_addr.lock().await;
            
            if let (Some(socket), Some(broadcast_addr)) = (socket_guard.as_ref(), broadcast_guard.as_ref()) {
                socket.send_to(&request, broadcast_addr).await?;

                // Collect I-Am responses (simplified - would need proper parsing)
                let mut devices = Vec::new();
                let mut buffer = vec![0u8; 1024];

                // Wait for responses with timeout
                let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(3);
                while tokio::time::Instant::now() < deadline {
                    match tokio::time::timeout(
                        std::time::Duration::from_millis(100),
                        socket.recv_from(&mut buffer)
                    ).await {
                        Ok(Ok((_size, _addr))) => {
                            // In real implementation, would parse I-Am response and extract device ID
                            // For now, just add a placeholder
                            devices.push(self.config.device_id);
                        }
                        _ => break,
                    }
                }

                log::debug!("Who-Is discovery found {} devices", devices.len());
                Ok(devices)
            } else {
                Err(Error::Connection("Socket or broadcast address not available".to_string()))
            }
        }
        #[cfg(not(feature = "bacnet"))]
        {
            Err(Error::NotSupported(
                "BACNet feature not enabled".to_string(),
            ))
        }
    }

    #[cfg(feature = "bacnet")]
    fn build_read_property_request(
        &self,
        _device_id: u32,
        object_type: BacnetObjectType,
        object_instance: u32,
        property: BacnetProperty,
    ) -> Result<Vec<u8>> {
        // Simplified BACNet/IP BVLC + NPDU + APDU construction
        let mut buffer = BytesMut::new();
        
        // BVLC Header (BACNet Virtual Link Control)
        buffer.put_u8(0x81); // Type: BACNet/IP
        buffer.put_u8(0x0A); // Function: Original-Unicast-NPDU
        buffer.put_u16(0); // Length placeholder
        
        // NPDU (Network Protocol Data Unit)
        buffer.put_u8(0x01); // Version
        buffer.put_u8(0x04); // Control flags: expecting reply
        
        // APDU (Application Protocol Data Unit) - Simplified Read Property
        buffer.put_u8(0x00); // PDU Type: Confirmed Request
        buffer.put_u8(0x05); // Service Choice: ReadProperty
        buffer.put_u8(0x0C); // Context tag: Object Identifier
        buffer.put_u32(((object_type as u32) << 22) | object_instance);
        buffer.put_u8(0x19); // Context tag: Property Identifier
        buffer.put_u8(property as u8);
        
        // Update length
        let len = buffer.len() as u16;
        buffer[2..4].copy_from_slice(&len.to_be_bytes());
        
        Ok(buffer.to_vec())
    }

    #[cfg(feature = "bacnet")]
    fn build_write_property_request(
        &self,
        _device_id: u32,
        object_type: BacnetObjectType,
        object_instance: u32,
        property: BacnetProperty,
        value: Bytes,
    ) -> Result<Vec<u8>> {
        // Simplified BACNet/IP Write Property request
        let mut buffer = BytesMut::new();
        
        // BVLC Header
        buffer.put_u8(0x81);
        buffer.put_u8(0x0A);
        buffer.put_u16(0);
        
        // NPDU
        buffer.put_u8(0x01);
        buffer.put_u8(0x04);
        
        // APDU - Write Property
        buffer.put_u8(0x00);
        buffer.put_u8(0x0F); // Service Choice: WriteProperty
        buffer.put_u8(0x0C);
        buffer.put_u32(((object_type as u32) << 22) | object_instance);
        buffer.put_u8(0x19);
        buffer.put_u8(property as u8);
        buffer.put_slice(&value);
        
        let len = buffer.len() as u16;
        buffer[2..4].copy_from_slice(&len.to_be_bytes());
        
        Ok(buffer.to_vec())
    }

    #[cfg(feature = "bacnet")]
    fn build_who_is_request(&self) -> Result<Vec<u8>> {
        // Simplified BACNet/IP Who-Is broadcast
        let mut buffer = BytesMut::new();
        
        // BVLC Header for broadcast
        buffer.put_u8(0x81);
        buffer.put_u8(0x0B); // Function: Original-Broadcast-NPDU
        buffer.put_u16(0);
        
        // NPDU
        buffer.put_u8(0x01);
        buffer.put_u8(0x20); // Broadcast
        buffer.put_u16(0xFFFF); // Destination network (global broadcast)
        buffer.put_u8(0xFF); // Broadcast MAC
        buffer.put_u8(0x00); // Hop count
        
        // APDU - Who-Is (unconfirmed service)
        buffer.put_u8(0x10); // PDU Type: Unconfirmed Request
        buffer.put_u8(0x08); // Service Choice: Who-Is
        
        let len = buffer.len() as u16;
        buffer[2..4].copy_from_slice(&len.to_be_bytes());
        
        Ok(buffer.to_vec())
    }
}

#[async_trait]
impl ProtocolHandler for BacnetHandler {
    async fn connect(&mut self) -> Result<()> {
        #[cfg(feature = "bacnet")]
        {
            // Bind to BACNet/IP port
            let bind_addr = format!("0.0.0.0:{}", self.config.port);
            let socket = UdpSocket::bind(&bind_addr).await?;
            socket.set_broadcast(true)?;

            // Set broadcast address
            let broadcast_addr = format!("255.255.255.255:{}", self.config.port)
                .parse()
                .map_err(|e| Error::Configuration(format!("Invalid broadcast address: {}", e)))?;

            *self.socket.lock().await = Some(socket);
            *self.broadcast_addr.lock().await = Some(broadcast_addr);
            *self.connected.lock().await = true;
            
            log::info!(
                "BACNet connected on port {}",
                self.config.port
            );
            Ok(())
        }
        #[cfg(not(feature = "bacnet"))]
        {
            Err(Error::NotSupported(
                "BACNet feature not enabled".to_string(),
            ))
        }
    }

    async fn disconnect(&mut self) -> Result<()> {
        #[cfg(feature = "bacnet")]
        {
            *self.socket.lock().await = None;
            *self.broadcast_addr.lock().await = None;
        }
        
        *self.connected.lock().await = false;
        log::info!("BACNet disconnected");
        Ok(())
    }

    async fn publish(&mut self, message: ProtocolMessage) -> Result<()> {
        // BACNet doesn't have a traditional publish concept
        // This could be used for COV (Change of Value) notifications
        let _ = message;
        Err(Error::NotSupported(
            "Publish not fully supported for BACNet. Use write_property() instead.".to_string(),
        ))
    }

    async fn subscribe(&mut self, topic: &str) -> Result<()> {
        // BACNet doesn't have a traditional subscribe concept
        // Could be used for COV (Change of Value) subscriptions in future
        let _ = topic;
        Err(Error::NotSupported(
            "Subscribe not fully supported for BACNet (COV subscriptions not yet implemented)".to_string(),
        ))
    }

    async fn poll_message(&mut self) -> Result<Option<ProtocolMessage>> {
        // First check queued messages
        let mut messages = self.messages.lock().await;
        if let Some(msg) = messages.pop_front() {
            return Ok(Some(msg));
        }
        drop(messages);

        #[cfg(feature = "bacnet")]
        {
            // Poll for incoming BACNet messages
            let socket_guard = self.socket.lock().await;
            if let Some(socket) = socket_guard.as_ref() {
                let mut buffer = vec![0u8; 1024];
                match tokio::time::timeout(
                    std::time::Duration::from_millis(10),
                    socket.recv_from(&mut buffer)
                ).await {
                    Ok(Ok((size, addr))) => {
                        // Parse BACNet message (simplified)
                        let msg = ProtocolMessage {
                            topic: format!("bacnet/{}", addr),
                            payload: Bytes::copy_from_slice(&buffer[..size]),
                            qos: 0,
                        };
                        Ok(Some(msg))
                    }
                    _ => Ok(None),
                }
            } else {
                Ok(None)
            }
        }
        #[cfg(not(feature = "bacnet"))]
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
    fn test_bacnet_config_default() {
        let config = BacnetConfig::default();
        assert_eq!(config.address, "127.0.0.1");
        assert_eq!(config.port, 47808);
        assert_eq!(config.device_id, 1234);
    }

    #[test]
    fn test_bacnet_handler_creation() {
        let config = BacnetConfig::default();
        let handler = BacnetHandler::new(config);
        assert!(!handler.is_connected());
    }

    #[test]
    fn test_bacnet_object_types() {
        assert_eq!(BacnetObjectType::AnalogInput as u32, 0);
        assert_eq!(BacnetObjectType::Device as u32, 8);
    }

    #[test]
    fn test_bacnet_properties() {
        assert_eq!(BacnetProperty::PresentValue as u32, 85);
        assert_eq!(BacnetProperty::ObjectName as u32, 77);
    }
}
