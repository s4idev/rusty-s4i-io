# Getting Started with rusty-s4i-io

This guide will help you get started with rusty-s4i-io quickly.

## Quick Start

### 1. Add to Your Project

Add rusty-s4i-io to your `Cargo.toml`:

```toml
[dependencies]
rusty-s4i-io = "0.1.0"
```

### 2. Create Your First Transport

Here's a simple TCP server example:

```rust
use rusty_s4i_io::{TransportManager, TransportConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a transport manager
    let mut manager = TransportManager::new();
    
    // Add a TCP server
    let config = TransportConfig::tcp_server("127.0.0.1:8080")?;
    let server_id = manager.add_transport(config).await?;
    
    // Connect
    manager.connect(&server_id).await?;
    
    println!("Server is running on port 8080");
    
    Ok(())
}
```

### 3. Send and Receive Data

```rust
use rusty_s4i_io::{TransportManager, TransportConfig, TransportId};
use bytes::Bytes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = TransportManager::new();
    
    // Create a UDP client
    let config = TransportConfig::udp("127.0.0.1:9000")?;
    let client_id = manager.add_transport(config).await?;
    manager.connect(&client_id).await?;
    
    // Send data
    let message = Bytes::from("Hello World!");
    manager.send(&client_id, &TransportId::Connection(0), message).await?;
    
    // Poll for events
    let events = manager.poll_events().await?;
    for (transport_id, event) in events {
        println!("Event from {}: {:?}", transport_id, event);
    }
    
    Ok(())
}
```

## Common Use Cases

### IoT Communication with MQTT

```rust
use rusty_s4i_io::protocol::{
    mqtt::{MqttConfig, MqttHandler},
    ProtocolHandler, ProtocolMessage,
};
use bytes::Bytes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = MqttConfig {
        broker_url: "mqtt://broker.hivemq.com:1883".to_string(),
        client_id: "my-device-001".to_string(),
        clean_session: true,
        ..Default::default()
    };
    
    let mut handler = MqttHandler::new(config);
    handler.connect().await?;
    
    // Subscribe to topics
    handler.subscribe("sensors/#").await?;
    
    // Publish data
    let message = ProtocolMessage {
        topic: "sensors/temperature".to_string(),
        payload: Bytes::from("22.5"),
        qos: 1,
    };
    handler.publish(message).await?;
    
    Ok(())
}
```

### Industrial Automation with Modbus

```rust
use rusty_s4i_io::protocol::{
    modbus::{ModbusConfig, ModbusHandler, ModbusProtocol},
    ProtocolHandler,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ModbusConfig {
        protocol: ModbusProtocol::Tcp,
        address: "192.168.1.100:502".to_string(),
        slave_id: 1,
    };
    
    let mut handler = ModbusHandler::new(config);
    handler.connect().await?;
    
    // Read sensor values from holding registers
    let temperatures = handler.read_holding_registers(1000, 5).await?;
    println!("Temperature sensors: {:?}", temperatures);
    
    // Write control value
    handler.write_single_register(2000, 100).await?;
    
    Ok(())
}
```

### Building Automation with BACNet

```rust
use rusty_s4i_io::protocol::{
    bacnet::{BacnetConfig, BacnetHandler, BacnetObjectType, BacnetProperty},
    ProtocolHandler,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = BacnetConfig {
        address: "192.168.1.1".to_string(),
        port: 47808,
        device_id: 12345,
        network_number: 0,
    };
    
    let mut handler = BacnetHandler::new(config);
    handler.connect().await?;
    
    // Discover BACNet devices on the network
    let devices = handler.who_is().await?;
    println!("Found {} BACNet devices", devices.len());
    
    // Read temperature from an analog input
    let value = handler.read_property(
        12345,
        BacnetObjectType::AnalogInput,
        1,
        BacnetProperty::PresentValue
    ).await?;
    
    Ok(())
}
```

## Feature Flags

Enable specific features based on your needs:

```toml
# Minimal - just TCP
rusty-s4i-io = { version = "0.1.0", default-features = false, features = ["tcp"] }

# IoT - MQTT over TCP
rusty-s4i-io = { version = "0.1.0", features = ["tcp", "mqtt"] }

# Industrial - Modbus
rusty-s4i-io = { version = "0.1.0", features = ["tcp", "serial", "modbus"] }

# Building automation - BACNet
rusty-s4i-io = { version = "0.1.0", features = ["udp", "bacnet"] }

# Everything
rusty-s4i-io = { version = "0.1.0", features = ["full"] }
```

## Error Handling

rusty-s4i-io provides a comprehensive error type:

```rust
use rusty_s4i_io::{TransportManager, TransportConfig, Error};

#[tokio::main]
async fn main() {
    let mut manager = TransportManager::new();
    
    match TransportConfig::tcp("invalid_address") {
        Ok(config) => {
            // Use config
        }
        Err(Error::Configuration(msg)) => {
            eprintln!("Configuration error: {}", msg);
        }
        Err(e) => {
            eprintln!("Other error: {}", e);
        }
    }
}
```

## Logging

Enable logging to see what's happening:

```toml
[dependencies]
rusty-s4i-io = "0.1.0"
env_logger = "0.11"
log = "0.4"
```

```rust
fn main() {
    env_logger::init();
    // Your code here
}
```

Run with logging:
```bash
RUST_LOG=debug cargo run
```

## Next Steps

- Read the [full documentation](https://docs.rs/rusty-s4i-io)
- Check out the [examples](https://github.com/s4idev/rusty-s4i-io/tree/main/examples)
- Review platform-specific notes in [PLATFORM.md](PLATFORM.md)
- Join the community and contribute!

## Common Patterns

### Multiple Transports

```rust
let mut manager = TransportManager::new();

// Add multiple transports
let tcp = manager.add_transport(TransportConfig::tcp_server("0.0.0.0:8080")?).await?;
let udp = manager.add_transport(TransportConfig::udp("0.0.0.0:9000")?).await?;

// Connect all
manager.connect(&tcp).await?;
manager.connect(&udp).await?;

// Poll events from all transports
loop {
    let events = manager.poll_events().await?;
    for (transport_id, event) in events {
        // Handle event
    }
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
}
```

### Graceful Shutdown

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut manager = TransportManager::new();
    let transport_id = manager.add_transport(config).await?;
    manager.connect(&transport_id).await?;
    
    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;
    
    // Clean shutdown
    manager.disconnect(&transport_id).await?;
    manager.remove_transport(&transport_id).await?;
    
    Ok(())
}
```

## Troubleshooting

### Connection Issues

If you can't connect:
1. Check firewall settings
2. Verify the address and port
3. Enable logging to see detailed errors
4. Check platform-specific requirements in PLATFORM.md

### Build Issues

If the build fails:
1. Check that you have the required system libraries (see PLATFORM.md)
2. Try building with default features only first
3. Enable features one at a time to identify the problem

### Performance Tips

1. Use feature flags to only include what you need
2. Adjust buffer sizes in `TransportOptions`
3. Use connection pooling for high-traffic scenarios
4. Consider using tokio's multi-threaded runtime for CPU-bound workloads

## Getting Help

- Check the [documentation](https://docs.rs/rusty-s4i-io)
- Review [examples](https://github.com/s4idev/rusty-s4i-io/tree/main/examples)
- Open an issue on [GitHub](https://github.com/s4idev/rusty-s4i-io/issues)
- Read the [PLATFORM.md](PLATFORM.md) for platform-specific help
