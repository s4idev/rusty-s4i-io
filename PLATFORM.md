# Platform Support Guide

This document provides detailed information about platform-specific considerations when using rusty-s4i-io.

## Supported Platforms

rusty-s4i-io is designed to work across multiple platforms:

- **Linux** (x86_64, ARM)
- **macOS** (Intel, Apple Silicon)
- **Windows** (x86_64)

## Transport-Specific Platform Notes

### TCP/IP and UDP
- **All platforms**: Fully supported
- No platform-specific considerations

### HTTP/HTTPS
- **All platforms**: Fully supported via `reqwest` crate
- TLS support may require system certificates

### Serial Port
- **Linux**: Requires read/write permissions on `/dev/tty*` devices
  - Add user to `dialout` group: `sudo usermod -a -G dialout $USER`
- **macOS**: Devices appear as `/dev/cu.*` or `/dev/tty.*`
- **Windows**: Use COM port names like `COM1`, `COM2`, etc.

### USB HID
- **Linux**: May require udev rules for device access
  - Example udev rule: `SUBSYSTEM=="usb", ATTRS{idVendor}=="XXXX", MODE="0666"`
- **macOS**: May require system permissions for USB access
- **Windows**: Usually works without additional configuration

### Bluetooth Low Energy (BLE)
- **Linux**: Requires `bluez` stack
  - Install: `sudo apt-get install bluez`
- **macOS**: Native Core Bluetooth support
- **Windows**: Requires Windows 10+ with Bluetooth 4.0+

### WebSocket
- **All platforms**: Fully supported
- No platform-specific considerations

### TLS/SSL
- **All platforms**: System certificate store used by default
- Custom certificates can be configured

## Compilation Notes

### Cross-Compilation

To cross-compile for different platforms:

```bash
# For Linux ARM (e.g., Raspberry Pi)
cargo build --target armv7-unknown-linux-gnueabihf

# For Windows from Linux
cargo build --target x86_64-pc-windows-gnu

# For macOS from Linux (requires osxcross)
cargo build --target x86_64-apple-darwin
```

### Feature Selection by Platform

Some features may have better support on certain platforms. You can use conditional features:

```toml
[target.'cfg(unix)'.dependencies]
rusty-s4i-io = { version = "0.1.0", features = ["serial"] }

[target.'cfg(windows)'.dependencies]
rusty-s4i-io = { version = "0.1.0", features = ["tcp", "udp"] }
```

## Testing on Different Platforms

### Linux
```bash
# Install dependencies
sudo apt-get install build-essential pkg-config libssl-dev

# Run tests
cargo test --all-features
```

### macOS
```bash
# Install Xcode command line tools
xcode-select --install

# Run tests
cargo test --all-features
```

### Windows
```powershell
# Install Visual Studio Build Tools
# Run tests
cargo test --all-features
```

## Performance Considerations

### Linux
- Best overall performance
- Direct access to system APIs
- Recommended for production deployments

### macOS
- Good performance
- Native BLE support is excellent
- Suitable for development and testing

### Windows
- Good performance for network protocols
- Serial port access may have slight overhead
- Recommended for Windows-specific applications

## Known Issues and Workarounds

### Linux
- Serial port permissions: See notes above about `dialout` group
- BLE may require root or special capabilities

### macOS
- USB HID may require privacy permissions in System Preferences
- First connection to BLE devices may prompt for permission

### Windows
- COM port names must match system names exactly
- Some antivirus software may interfere with network operations

## Security Considerations

### All Platforms
- Always use TLS for sensitive data transmission
- Validate certificates in production
- Keep dependencies updated for security patches

### Linux
- Use firewalls to restrict network access
- Consider using SELinux or AppArmor for additional security

### macOS
- Use app sandboxing when appropriate
- Request minimal permissions

### Windows
- Use Windows Firewall to control network access
- Consider User Account Control (UAC) implications

## Debugging Tips

### Enable Logging
```rust
env_logger::init();
```

Set log level:
```bash
RUST_LOG=debug cargo run
```

### Platform-Specific Debugging

**Linux:**
```bash
# Monitor USB devices
lsusb

# Monitor serial ports
dmesg | grep tty

# Test network connectivity
netstat -tulpn
```

**macOS:**
```bash
# Monitor USB devices
system_profiler SPUSBDataType

# Monitor serial ports
ls /dev/cu.*
```

**Windows:**
```powershell
# List COM ports
mode

# Monitor network connections
netstat -an
```

## Contributing Platform Support

When adding platform-specific code:

1. Use conditional compilation:
   ```rust
   #[cfg(target_os = "linux")]
   fn linux_specific() { }
   
   #[cfg(target_os = "macos")]
   fn macos_specific() { }
   
   #[cfg(target_os = "windows")]
   fn windows_specific() { }
   ```

2. Add platform-specific tests:
   ```rust
   #[cfg(target_os = "linux")]
   #[test]
   fn test_linux_feature() { }
   ```

3. Document any platform limitations in code and this file

## Support

For platform-specific issues, please include the following in bug reports:
- Operating system and version
- Rust version (`rustc --version`)
- Cargo version (`cargo --version`)
- Relevant system information (architecture, kernel version, etc.)
