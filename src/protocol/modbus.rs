//! Modbus protocol implementation

use crate::error::{Error, Result};
use crate::protocol::{ProtocolHandler, ProtocolMessage};
use async_trait::async_trait;
use bytes::Bytes;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(feature = "modbus")]
use tokio_modbus::client::tcp::Context;
#[cfg(feature = "modbus")]
use tokio_modbus::prelude::*;

/// Modbus protocol type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModbusProtocol {
    Tcp,
    Rtu,
}

/// Modbus client configuration
#[derive(Debug, Clone)]
pub struct ModbusConfig {
    pub protocol: ModbusProtocol,
    pub address: String,
    pub slave_id: u8,
}

impl Default for ModbusConfig {
    fn default() -> Self {
        Self {
            protocol: ModbusProtocol::Tcp,
            address: "127.0.0.1:502".to_string(),
            slave_id: 1,
        }
    }
}

/// Modbus function codes
#[derive(Debug, Clone, Copy)]
pub enum ModbusFunction {
    ReadCoils = 0x01,
    ReadDiscreteInputs = 0x02,
    ReadHoldingRegisters = 0x03,
    ReadInputRegisters = 0x04,
    WriteSingleCoil = 0x05,
    WriteSingleRegister = 0x06,
    WriteMultipleCoils = 0x0F,
    WriteMultipleRegisters = 0x10,
}

/// Modbus protocol handler
pub struct ModbusHandler {
    config: ModbusConfig,
    #[cfg(feature = "modbus")]
    context: Arc<Mutex<Option<Context>>>,
    connected: Arc<Mutex<bool>>,
    messages: Arc<Mutex<VecDeque<ProtocolMessage>>>,
}

impl ModbusHandler {
    /// Create a new Modbus handler
    pub fn new(config: ModbusConfig) -> Self {
        Self {
            config,
            #[cfg(feature = "modbus")]
            context: Arc::new(Mutex::new(None)),
            connected: Arc::new(Mutex::new(false)),
            messages: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Read holding registers
    pub async fn read_holding_registers(&mut self, address: u16, count: u16) -> Result<Vec<u16>> {
        #[cfg(feature = "modbus")]
        {
            use tokio_modbus::client::Reader;
            let mut ctx_guard = self.context.lock().await;
            if let Some(ctx) = ctx_guard.as_mut() {
                let result: Vec<u16> = ctx
                    .read_holding_registers(address, count)
                    .await
                    .map_err(|e| Error::Protocol(format!("Modbus read failed: {}", e)))?;
                
                log::debug!("Read {} holding registers from address {}", count, address);
                Ok(result)
            } else {
                Err(Error::Connection("Not connected to Modbus server".to_string()))
            }
        }
        #[cfg(not(feature = "modbus"))]
        {
            let _ = (address, count);
            Err(Error::NotSupported("Modbus feature not enabled".to_string()))
        }
    }

    /// Write single register
    pub async fn write_single_register(&mut self, address: u16, value: u16) -> Result<()> {
        #[cfg(feature = "modbus")]
        {
            use tokio_modbus::client::Writer;
            let mut ctx_guard = self.context.lock().await;
            if let Some(ctx) = ctx_guard.as_mut() {
                match ctx.write_single_register(address, value).await {
                    Ok(_) => {
                        log::debug!("Wrote value {} to register {}", value, address);
                        Ok(())
                    }
                    Err(e) => Err(Error::Protocol(format!("Modbus write failed: {}", e))),
                }
            } else {
                Err(Error::Connection("Not connected to Modbus server".to_string()))
            }
        }
        #[cfg(not(feature = "modbus"))]
        {
            let _ = (address, value);
            Err(Error::NotSupported("Modbus feature not enabled".to_string()))
        }
    }

    /// Read input registers
    pub async fn read_input_registers(&mut self, address: u16, count: u16) -> Result<Vec<u16>> {
        #[cfg(feature = "modbus")]
        {
            use tokio_modbus::client::Reader;
            let mut ctx_guard = self.context.lock().await;
            if let Some(ctx) = ctx_guard.as_mut() {
                let result: Vec<u16> = ctx
                    .read_input_registers(address, count)
                    .await
                    .map_err(|e| Error::Protocol(format!("Modbus read failed: {}", e)))?;
                
                log::debug!("Read {} input registers from address {}", count, address);
                Ok(result)
            } else {
                Err(Error::Connection("Not connected to Modbus server".to_string()))
            }
        }
        #[cfg(not(feature = "modbus"))]
        {
            let _ = (address, count);
            Err(Error::NotSupported("Modbus feature not enabled".to_string()))
        }
    }

    /// Read coils
    pub async fn read_coils(&mut self, address: u16, count: u16) -> Result<Vec<bool>> {
        #[cfg(feature = "modbus")]
        {
            use tokio_modbus::client::Reader;
            let mut ctx_guard = self.context.lock().await;
            if let Some(ctx) = ctx_guard.as_mut() {
                let result: Vec<bool> = ctx
                    .read_coils(address, count)
                    .await
                    .map_err(|e| Error::Protocol(format!("Modbus read failed: {}", e)))?;
                
                log::debug!("Read {} coils from address {}", count, address);
                Ok(result)
            } else {
                Err(Error::Connection("Not connected to Modbus server".to_string()))
            }
        }
        #[cfg(not(feature = "modbus"))]
        {
            let _ = (address, count);
            Err(Error::NotSupported("Modbus feature not enabled".to_string()))
        }
    }

    /// Write single coil
    pub async fn write_single_coil(&mut self, address: u16, value: bool) -> Result<()> {
        #[cfg(feature = "modbus")]
        {
            use tokio_modbus::client::Writer;
            let mut ctx_guard = self.context.lock().await;
            if let Some(ctx) = ctx_guard.as_mut() {
                match ctx.write_single_coil(address, value).await {
                    Ok(_) => {
                        log::debug!("Wrote value {} to coil {}", value, address);
                        Ok(())
                    }
                    Err(e) => Err(Error::Protocol(format!("Modbus write failed: {}", e))),
                }
            } else {
                Err(Error::Connection("Not connected to Modbus server".to_string()))
            }
        }
        #[cfg(not(feature = "modbus"))]
        {
            let _ = (address, value);
            Err(Error::NotSupported("Modbus feature not enabled".to_string()))
        }
    }
}

#[async_trait]
impl ProtocolHandler for ModbusHandler {
    async fn connect(&mut self) -> Result<()> {
        #[cfg(feature = "modbus")]
        {
            match self.config.protocol {
                ModbusProtocol::Tcp => {
                    let socket_addr = self.config.address.parse()
                        .map_err(|e| Error::Configuration(format!("Invalid address: {}", e)))?;
                    
                    let ctx = tokio_modbus::client::tcp::connect_slave(socket_addr, Slave(self.config.slave_id))
                        .await
                        .map_err(|e| Error::Connection(format!("Modbus TCP connection failed: {}", e)))?;
                    
                    *self.context.lock().await = Some(ctx);
                    *self.connected.lock().await = true;
                    
                    log::info!("Modbus TCP connected to {}", self.config.address);
                    Ok(())
                }
                ModbusProtocol::Rtu => {
                    // Modbus RTU would require a different context type
                    // For now, we only support TCP
                    Err(Error::NotSupported("Modbus RTU not yet supported. Use Modbus TCP instead.".to_string()))
                }
            }
        }
        #[cfg(not(feature = "modbus"))]
        {
            Err(Error::NotSupported("Modbus feature not enabled".to_string()))
        }
    }

    async fn disconnect(&mut self) -> Result<()> {
        #[cfg(feature = "modbus")]
        {
            *self.context.lock().await = None;
        }
        
        *self.connected.lock().await = false;
        log::info!("Modbus disconnected");
        Ok(())
    }

    async fn publish(&mut self, message: ProtocolMessage) -> Result<()> {
        // Modbus doesn't have a traditional publish concept
        // This could be used to write data to registers
        let _ = message;
        Err(Error::NotSupported("Publish not supported for Modbus. Use write_single_register() instead.".to_string()))
    }

    async fn subscribe(&mut self, topic: &str) -> Result<()> {
        // Modbus doesn't have a subscribe concept
        let _ = topic;
        Err(Error::NotSupported("Subscribe not supported for Modbus".to_string()))
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
    fn test_modbus_config_default() {
        let config = ModbusConfig::default();
        assert_eq!(config.protocol, ModbusProtocol::Tcp);
        assert_eq!(config.address, "127.0.0.1:502");
        assert_eq!(config.slave_id, 1);
    }

    #[test]
    fn test_modbus_handler_creation() {
        let config = ModbusConfig::default();
        let handler = ModbusHandler::new(config);
        assert!(!handler.is_connected());
    }

    #[test]
    fn test_modbus_function_codes() {
        assert_eq!(ModbusFunction::ReadHoldingRegisters as u8, 0x03);
        assert_eq!(ModbusFunction::WriteSingleRegister as u8, 0x06);
    }
}
