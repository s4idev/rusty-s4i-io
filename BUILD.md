# Building rusty-s4i-io

This guide provides detailed information about building rusty-s4i-io on different platforms with various feature combinations.

## Quick Start

### Minimal Build (TCP/UDP only)

Works on all platforms without additional dependencies:

```bash
cargo build
cargo test
```

### Network-Only Build

For applications that only need network protocols:

```bash
cargo build --features tcp,udp,http,websocket,tls,mqtt,modbus,bacnet
cargo test --features tcp,udp,http,websocket,tls,mqtt,modbus,bacnet
```

This combination doesn't require platform-specific system libraries and is recommended for most use cases.

## Feature Flags

rusty-s4i-io uses Cargo feature flags to enable optional functionality:

### Transport Features

| Feature | Description | System Dependencies |
|---------|-------------|-------------------|
| `tcp` | TCP/IP transport (default) | None |
| `udp` | UDP transport (default) | None |
| `http` | HTTP/HTTPS transport | None |
| `websocket` | WebSocket transport | None |
| `tls` | TLS/SSL support | None |
| `serial` | Serial port (RS-232/RS-485) | Platform-specific (see below) |
| `usb` | USB HID support | `libudev` (Linux) |
| `ble` | Bluetooth Low Energy | `libdbus` (Linux) |

### Protocol Features

| Feature | Description | System Dependencies |
|---------|-------------|-------------------|
| `mqtt` | MQTT protocol | None |
| `modbus` | Modbus TCP/RTU | None |
| `bacnet` | BACNet/IP protocol | None |

### Combined Features

| Feature | Description |
|---------|-------------|
| `all-transports` | All transport features |
| `all-protocols` | All protocol features |
| `full` | All features (requires all dependencies) |

## Platform-Specific Build Instructions

### Linux (Ubuntu/Debian)

#### Minimal (Default Features)

```bash
cargo build
```

#### With HTTP and Protocols

```bash
cargo build --features http,mqtt,modbus,bacnet
```

#### With Serial Port Support

```bash
# Install dependencies
sudo apt-get install -y libudev-dev

# Build
cargo build --features serial
```

#### With USB Support

```bash
# Install dependencies
sudo apt-get install -y libudev-dev pkg-config

# Build
cargo build --features usb
```

#### With BLE Support

```bash
# Install dependencies
sudo apt-get install -y libdbus-1-dev pkg-config

# Build
cargo build --features ble
```

#### All Features

```bash
# Install all dependencies
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libudev-dev \
    libdbus-1-dev

# Build
cargo build --all-features
```

### Linux (Fedora/RHEL)

#### All Dependencies

```bash
# Install dependencies
sudo dnf install -y \
    gcc \
    pkg-config \
    openssl-devel \
    systemd-devel \
    dbus-devel

# Build
cargo build --all-features
```

### macOS

#### Default Build

```bash
cargo build
```

#### With Serial Port

```bash
# No additional dependencies needed
cargo build --features serial
```

#### With BLE

```bash
# No additional dependencies needed (uses Core Bluetooth)
cargo build --features ble
```

#### With HTTP and Protocols

```bash
cargo build --features http,mqtt,modbus,bacnet
```

#### Note on USB

USB HID support on macOS works but may require additional permissions. Your application may need to be code-signed or users may need to grant permissions in System Preferences.

### Windows

#### Default Build

```bash
cargo build
```

#### With Serial Port

```bash
# No additional dependencies beyond Visual Studio Build Tools
cargo build --features serial
```

#### With HTTP and Protocols

```bash
cargo build --features http,mqtt,modbus,bacnet
```

#### Prerequisites

Ensure you have Visual Studio Build Tools installed with C++ support:
1. Download from https://visualstudio.microsoft.com/downloads/
2. Install "Desktop development with C++" workload
3. Restart your terminal/PowerShell

## Cross-Compilation

### Linux to Windows

```bash
# Install target
rustup target add x86_64-pc-windows-gnu

# Install mingw
sudo apt-get install -y mingw-w64

# Build
cargo build --target x86_64-pc-windows-gnu
```

### Linux to ARM (e.g., Raspberry Pi)

```bash
# Install target
rustup target add armv7-unknown-linux-gnueabihf

# Install cross-compiler
sudo apt-get install -y gcc-arm-linux-gnueabihf

# Build
cargo build --target armv7-unknown-linux-gnueabihf
```

## Docker Build

For a consistent build environment, you can use Docker:

### Create Dockerfile

```dockerfile
FROM rust:1.70

WORKDIR /app

# Install Linux dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libudev-dev \
    libdbus-1-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy source
COPY . .

# Build with all features
RUN cargo build --release --all-features

CMD ["cargo", "test", "--release"]
```

### Build and Run

```bash
docker build -t rusty-s4i-io .
docker run rusty-s4i-io
```

## Optimized Builds

### Release Build

```bash
cargo build --release
```

### Size-Optimized Build

Add to `Cargo.toml`:

```toml
[profile.release]
opt-level = 'z'     # Optimize for size
lto = true          # Enable Link Time Optimization
codegen-units = 1   # Better optimization, slower compile
strip = true        # Strip symbols
```

Then build:

```bash
cargo build --release
```

### Performance-Optimized Build

```toml
[profile.release]
opt-level = 3       # Maximum optimization
lto = true
codegen-units = 1
```

## Build Troubleshooting

### Issue: `pkg-config` not found

**Solution:**
```bash
# Ubuntu/Debian
sudo apt-get install pkg-config

# macOS
brew install pkg-config

# Windows
# pkg-config is not typically needed on Windows
```

### Issue: `libudev` not found (Linux)

**Solution:**
```bash
sudo apt-get install libudev-dev
```

### Issue: `libdbus` not found (Linux)

**Solution:**
```bash
sudo apt-get install libdbus-1-dev
```

### Issue: OpenSSL errors

**Solution:**
```bash
# Ubuntu/Debian
sudo apt-get install libssl-dev

# macOS
brew install openssl
export OPENSSL_DIR=$(brew --prefix openssl)

# Windows
# Usually works out of the box with Visual Studio
```

### Issue: Linking errors on Windows

**Solution:**
- Ensure Visual Studio Build Tools are installed
- Try running from Visual Studio Developer Command Prompt
- Check that you have the latest Windows SDK

### Issue: Permission denied for serial port (Linux)

**Solution:**
```bash
# Add user to dialout group
sudo usermod -a -G dialout $USER

# Log out and log back in
```

## CI/CD Integration

### GitHub Actions

See `.github/workflows/ci.yml` for our CI configuration.

### GitLab CI

Example `.gitlab-ci.yml`:

```yaml
image: rust:latest

stages:
  - test
  - build

test:
  stage: test
  before_script:
    - apt-get update && apt-get install -y pkg-config libssl-dev
  script:
    - cargo test --verbose

build:
  stage: build
  script:
    - cargo build --release
  artifacts:
    paths:
      - target/release/rusty-s4i-io
```

## Minimum Supported Rust Version (MSRV)

rusty-s4i-io requires Rust 1.70.0 or later.

To check if your Rust version is compatible:

```bash
rustc --version
```

To update Rust:

```bash
rustup update
```

## Feature Combination Guide

### IoT Applications

```bash
cargo build --features tcp,mqtt
```

### Industrial Automation

```bash
cargo build --features tcp,serial,modbus
```

### Building Automation

```bash
cargo build --features udp,bacnet
```

### Web Services

```bash
cargo build --features http,websocket,tls
```

### Embedded Systems

For embedded targets, you might want minimal features:

```bash
cargo build --no-default-features --features tcp
```

## Build Performance Tips

1. **Use cargo-build cache:** Enable `sccache` for faster rebuilds
   ```bash
   cargo install sccache
   export RUSTC_WRAPPER=sccache
   ```

2. **Parallel compilation:** Set in `.cargo/config.toml`:
   ```toml
   [build]
   jobs = 4  # Adjust based on CPU cores
   ```

3. **Incremental compilation:** Already enabled by default for debug builds

4. **Use feature flags wisely:** Only include features you need to reduce compile time

## Additional Resources

- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [Platform-Specific Notes](PLATFORM.md)
- [Contributing Guide](CONTRIBUTING.md)
- [Main README](README.md)
