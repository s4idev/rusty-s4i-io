# rusty-s4i-io

A comprehensive Rust library for managing multiple transport protocols with a unified framework, supporting TCP/IP, UDP, HTTP/HTTPS, serial port, USB HID, BLE, WebSocket, and SSL/TLS. Additionally provides application layer protocol support for MQTT, Modbus, and BACNet/IP.

[![Crates.io](https://img.shields.io/crates/v/rusty-s4i-io.svg)](https://crates.io/crates/rusty-s4i-io)
[![Documentation](https://docs.rs/rusty-s4i-io/badge.svg)](https://docs.rs/rusty-s4i-io)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

`rusty-s4i-io` is a comprehensive Rust library designed to streamline the integration of various transport protocols and application layer protocols. By providing a unified framework, it simplifies the development of networked and industrial automation applications across multiple platforms (Linux, macOS, Windows).

## Features

### Transport Layer
- **TCP/IP** - Client and server support with async I/O
- **UDP** - Unicast and broadcast support
- **HTTP/HTTPS** - REST API client support
- **Serial Port** - RS-232/RS-485 communication
- **USB HID** - Human Interface Device support
- **Bluetooth Low Energy (BLE)** - Wireless communication
- **WebSocket** - Full-duplex communication
- **SSL/TLS** - Secure socket layer

### Application Layer Protocols
- **MQTT** - Message Queue Telemetry Transport for IoT
- **Modbus** - Industrial automation protocol (TCP and RTU)
- **BACNet/IP** - Building automation and control networks

### Cross-Platform Support
- **Linux** - Full support for all features
- **macOS** - Full support for all features
- **Windows** - Full support for all features

Platform-specific features are handled through conditional compilation using Cargo feature flags.

## Installation

Add `rusty-s4i-io` to your `Cargo.toml`:

```toml
[dependencies]
# Default features (TCP and UDP)
rusty-s4i-io = "0.1.0"

# With specific transports
rusty-s4i-io = { version = "0.1.0", features = ["tcp", "udp", "http"] }

# With application protocols
rusty-s4i-io = { version = "0.1.0", features = ["mqtt", "modbus", "bacnet"] }

# With all features
rusty-s4i-io = { version = "0.1.0", features = ["full"] }
```

### Available Features

**Transport Features:**
- `tcp` - TCP/IP support (default)
- `udp` - UDP support (default)
- `http` - HTTP/HTTPS support
- `websocket` - WebSocket support
- `tls` - TLS/SSL support
- `serial` - Serial port support
- `usb` - USB HID support
- `ble` - Bluetooth Low Energy support
- `all-transports` - Enable all transport features

**Protocol Features:**
- `mqtt` - MQTT protocol support
- `modbus` - Modbus protocol support
- `bacnet` - BACNet/IP protocol support
- `all-protocols` - Enable all protocol features

**Combined Features:**
- `full` - Enable all transports and protocols

## Usage

### Basic TCP Example

```rust
use rusty_s4i_io::{TransportManager, TransportConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = TransportManager::new();
    
    // Create TCP server
    let config = TransportConfig::tcp_server("127.0.0.1:8080")?;
    let server_id = manager.add_transport(config).await?;
    manager.connect(&server_id).await?;
    
    println!("Server listening on port 8080");
    
    Ok(())
}
```

### UDP Communication Example

```rust
use rusty_s4i_io::{TransportManager, TransportConfig, TransportId};
use bytes::Bytes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = TransportManager::new();
    
    // Create UDP transport
    let config = TransportConfig::udp("127.0.0.1:9000")?;
    let transport_id = manager.add_transport(config).await?;
    manager.connect(&transport_id).await?;
    
    // Send data
    let data = Bytes::from("Hello UDP!");
    manager.send(&transport_id, &TransportId::Connection(0), data).await?;
    
    Ok(())
}
```

### MQTT Example

```rust
use rusty_s4i_io::protocol::{
    mqtt::{MqttConfig, MqttHandler},
    ProtocolHandler,
    ProtocolMessage,
};
use bytes::Bytes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = MqttConfig {
        broker_url: "mqtt://localhost:1883".to_string(),
        client_id: "my-client".to_string(),
        ..Default::default()
    };
    
    let mut handler = MqttHandler::new(config);
    handler.connect().await?;
    handler.subscribe("sensors/#").await?;
    
    // Publish message
    let msg = ProtocolMessage {
        topic: "sensors/temperature".to_string(),
        payload: Bytes::from("25.5"),
        qos: 1,
    };
    handler.publish(msg).await?;
    
    Ok(())
}
```

### Modbus Example

```rust
use rusty_s4i_io::protocol::{
    modbus::{ModbusConfig, ModbusHandler, ModbusProtocol},
    ProtocolHandler,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ModbusConfig {
        protocol: ModbusProtocol::Tcp,
        address: "127.0.0.1:502".to_string(),
        slave_id: 1,
    };
    
    let mut handler = ModbusHandler::new(config);
    handler.connect().await?;
    
    // Read holding registers
    let values = handler.read_holding_registers(100, 10).await?;
    println!("Read {} registers", values.len());
    
    // Write single register
    handler.write_single_register(100, 42).await?;
    
    Ok(())
}
```

### BACNet/IP Example

```rust
use rusty_s4i_io::protocol::{
    bacnet::{BacnetConfig, BacnetHandler, BacnetObjectType, BacnetProperty},
    ProtocolHandler,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = BacnetConfig::default();
    let mut handler = BacnetHandler::new(config);
    handler.connect().await?;
    
    // Discover devices
    let devices = handler.who_is().await?;
    
    // Read property
    let value = handler.read_property(
        1234,
        BacnetObjectType::AnalogInput,
        0,
        BacnetProperty::PresentValue
    ).await?;
    
    Ok(())
}
```

## Architecture

### Module Structure

- **lib.rs**: Entry point of the library, exposing the main API
- **error.rs**: Error types and result handling
- **manager.rs**: TransportManager for managing multiple transports
- **transport/**: Transport layer implementations
  - **common.rs**: Common traits and types
  - **tcp.rs**: TCP transport implementation
  - **udp.rs**: UDP transport implementation
  - **http.rs**: HTTP/HTTPS transport implementation
  - **serial.rs**: Serial port transport implementation
  - **usb.rs**: USB HID transport implementation
  - **ble.rs**: Bluetooth Low Energy transport implementation
  - **websocket.rs**: WebSocket transport implementation
  - **tls.rs**: TLS/SSL transport implementation
- **protocol/**: Application layer protocol implementations
  - **common.rs**: Common protocol traits
  - **mqtt.rs**: MQTT protocol implementation
  - **modbus.rs**: Modbus protocol implementation
  - **bacnet.rs**: BACNet/IP protocol implementation

### Design Principles

1. **Unified Interface**: All transports implement the `TransportService` trait
2. **Async/Await**: Built on Tokio for efficient async I/O
3. **Type Safety**: Leverages Rust's type system for compile-time guarantees
4. **Modularity**: Feature flags allow selective compilation
5. **Cross-Platform**: Conditional compilation for platform-specific code

## Testing

Run the test suite:

```bash
# Run all tests with default features
cargo test

# Run tests with all features
cargo test --all-features

# Run specific transport tests
cargo test --features tcp,udp

# Run protocol tests
cargo test --features mqtt,modbus,bacnet
```

## Examples

See the `examples/` directory for complete working examples:

- `tcp_example.rs` - TCP client/server
- `udp_example.rs` - UDP communication
- `mqtt_example.rs` - MQTT pub/sub
- `modbus_example.rs` - Modbus communication
- `bacnet_example.rs` - BACNet/IP operations

Run examples with:

```bash
cargo run --example tcp_example --features tcp
cargo run --example mqtt_example --features mqtt
cargo run --example modbus_example --features modbus
```

## Platform-Specific Notes

### Linux
All features are fully supported. Serial port access may require appropriate permissions.

### macOS
All features are fully supported. USB HID may require system permissions.

### Windows
All features are supported. Serial port COM names use Windows format (e.g., `COM1`).

## Contributing

Contributions are welcome! Please follow these guidelines:

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Roadmap

- [ ] Add more transport protocols (QUIC, gRPC)
- [ ] Enhance protocol implementations with full feature sets
- [ ] Add TLS certificate management utilities
- [ ] Implement connection pooling
- [ ] Add metrics and monitoring support
- [ ] Expand cross-platform testing

## Acknowledgments

Built with:
- [Tokio](https://tokio.rs/) - Async runtime
- [bytes](https://github.com/tokio-rs/bytes) - Byte buffer utilities
- [serde](https://serde.rs/) - Serialization framework

For industrial automation protocols:
- [rumqttc](https://github.com/bytebeamio/rumqtt) - MQTT client
- [tokio-modbus](https://github.com/slowtec/tokio-modbus) - Modbus protocol
