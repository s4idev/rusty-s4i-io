//! Protocol integration tests

#[cfg(feature = "mqtt")]
mod mqtt_tests {
    use rusty_s4i_io::protocol::mqtt::{MqttConfig, MqttHandler};
    use rusty_s4i_io::protocol::ProtocolHandler;

    #[tokio::test]
    async fn test_mqtt_handler_creation() {
        let config = MqttConfig::default();
        let handler = MqttHandler::new(config);
        assert!(!handler.is_connected());
    }
}

#[cfg(feature = "modbus")]
mod modbus_tests {
    use rusty_s4i_io::protocol::modbus::{ModbusConfig, ModbusHandler};
    use rusty_s4i_io::protocol::ProtocolHandler;

    #[tokio::test]
    async fn test_modbus_handler_creation() {
        let config = ModbusConfig::default();
        let handler = ModbusHandler::new(config);
        assert!(!handler.is_connected());
    }
}

#[cfg(feature = "bacnet")]
mod bacnet_tests {
    use rusty_s4i_io::protocol::bacnet::{BacnetConfig, BacnetHandler};
    use rusty_s4i_io::protocol::ProtocolHandler;

    #[tokio::test]
    async fn test_bacnet_handler_creation() {
        let config = BacnetConfig::default();
        let handler = BacnetHandler::new(config);
        assert!(!handler.is_connected());
    }
}
