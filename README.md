# rusty-s4i-io
A comprehensive Rust library for managing multiple transport protocols with a unified framework, supporting TCP/IP, UDP, HTTP/HTTPS, serial port, USB HID, BLE, WebSocket, and SSL/TLS.

`rusty-s4i-io` is a comprehensive Rust library designed to streamline the integration of various transport protocols. By providing a unified framework for TCP/IP, UDP, HTTP/HTTPS, serial port, USB HID, BLE, WebSocket, and SSL/TLS, it simplifies the development of networked applications.

## Features

- **TCP/IP**
- **UDP Unicast/Broadcast**
- **HTTP/HTTPS**
- **Serial Port**
- **USB HID**
- **Bluetooth Low Energy (BLE)**
- **WebSocket**
- **Plain and Secure Sockets (SSL/TLS)**

## Installation

Add `rusty-s4i-io` to your `Cargo.toml`:

```toml
[dependencies]
rusty-s4i-io = "0.1.0"
```

## Usage

### Example

```rust
use rusty_s4i_io::{TransportManager, TransportService, TransportEvent, TransportId};

#[tokio::main]
async fn main() {
    let urls = vec![
        String::from("tcp://localhost:8080"),
        // Add other URLs...
    ];
    
    let mut manager = TransportManager::new(urls);
    manager.start();

    // Run your application logic...
    
    manager.stop();
}
```

### TransportManager

The `TransportManager` struct is responsible for managing multiple transport services. It initializes the appropriate transport services based on the provided URLs and handles their lifecycle.

### TransportService Trait

The `TransportService` trait defines the interface for all transport services. It includes methods for connecting, disconnecting, sending data, and handling events.

### TransportEvent Enum

The `TransportEvent` enum represents the different events that can occur during the lifecycle of a transport service.

### TransportId Enum

The `TransportId` enum is used to identify specific transports or to broadcast to all clients of a server transport.

## Module Structure

- **lib.rs**: Entry point of the library, exposing the main API.
- **transport/mod.rs**: Main module to include all sub-modules and define common traits and structures.
- **transport/common.rs**: Defines common traits and enums used across different transport modules.
- **transport/tcp.rs**: Implementation for TCP transport.
- **transport/udp.rs**: Implementation for UDP transport.
- **transport/http.rs**: Implementation for HTTP/HTTPS transport.
- **transport/serial.rs**: Implementation for serial port transport.
- **transport/usb.rs**: Implementation for USB HID transport.
- **transport/ble.rs**: Implementation for Bluetooth Low Energy (BLE) transport.
- **transport/websocket.rs**: Implementation for WebSocket transport.
- **transport/tls.rs**: Implementation for secure sockets (SSL/TLS) transport.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request on GitHub.

## License

This project is licensed under the MIT License. See the `LICENSE` file for details.

This `README.md` provides a concise description along with an overview of the crate, installation instructions, a usage example, and detailed information about the key components and module structure.
