# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2024-02-09

### Added
- Initial release of rusty-s4i-io
- Transport layer support:
  - TCP/IP (client and server)
  - UDP (unicast and broadcast)
  - HTTP/HTTPS
  - Serial Port
  - USB HID
  - Bluetooth Low Energy (BLE)
  - WebSocket
  - SSL/TLS
- Application layer protocol support:
  - MQTT (Message Queue Telemetry Transport)
  - Modbus (TCP and RTU)
  - BACNet/IP
- Cross-platform support (Linux, macOS, Windows)
- Unified `TransportService` trait for all transports
- `TransportManager` for managing multiple transports
- Comprehensive error handling with custom `Error` type
- Async/await support using Tokio
- Feature flags for selective compilation
- Unit tests for all modules
- Integration tests
- Examples for each transport and protocol
- Full documentation

### Features
- `tcp` - TCP/IP support (enabled by default)
- `udp` - UDP support (enabled by default)
- `http` - HTTP/HTTPS support
- `websocket` - WebSocket support
- `tls` - TLS/SSL support
- `serial` - Serial port support
- `usb` - USB HID support
- `ble` - BLE support
- `mqtt` - MQTT protocol
- `modbus` - Modbus protocol
- `bacnet` - BACNet/IP protocol
- `all-transports` - All transport features
- `all-protocols` - All protocol features
- `full` - All features

[0.1.0]: https://github.com/s4idev/rusty-s4i-io/releases/tag/v0.1.0
