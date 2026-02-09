# Contributing to rusty-s4i-io

Thank you for your interest in contributing to rusty-s4i-io! This document provides guidelines and information for contributors.

## Development Setup

### Prerequisites

- Rust 1.70.0 or later
- cargo
- Platform-specific dependencies (see below)

### Platform-Specific Dependencies

#### Linux

For full feature support on Linux, you'll need:

```bash
# Ubuntu/Debian
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libudev-dev \
    libdbus-1-dev

# Fedora/RHEL
sudo dnf install -y \
    gcc \
    pkg-config \
    openssl-devel \
    systemd-devel \
    dbus-devel
```

#### macOS

Most features work out of the box. For full support:

```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install Homebrew (if not already installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

#### Windows

Install Visual Studio Build Tools or Visual Studio with C++ support:
- Download from: https://visualstudio.microsoft.com/downloads/
- Select "Desktop development with C++" workload

## Building

### Build with default features (TCP and UDP only)

```bash
cargo build
```

### Build with specific features

```bash
# Network transports only
cargo build --features tcp,udp,http,websocket,tls

# With protocols
cargo build --features tcp,udp,mqtt,modbus,bacnet

# Platform-specific features (requires system dependencies)
cargo build --features serial,usb,ble
```

### Build all features

**Note:** This requires all system dependencies to be installed.

```bash
cargo build --all-features
```

## Testing

### Run default tests

```bash
cargo test
```

### Test specific features

```bash
# Test a transport
cargo test --no-default-features --features tcp

# Test a protocol
cargo test --no-default-features --features tcp,mqtt

# Test platform feature (requires dependencies)
cargo test --features serial
```

### Run all tests (requires all dependencies)

```bash
cargo test --all-features
```

## Code Quality

### Formatting

We use `rustfmt` for code formatting:

```bash
cargo fmt

# Check formatting without making changes
cargo fmt -- --check
```

### Linting

We use `clippy` for linting:

```bash
cargo clippy

# For CI-level strictness
cargo clippy -- -D warnings
```

## Documentation

### Build documentation

```bash
cargo doc --open
```

### Test documentation examples

```bash
cargo test --doc
```

## Feature Development Guidelines

### Adding a New Transport

1. Create a new module in `src/transport/`
2. Implement the `TransportService` trait
3. Add feature flag in `Cargo.toml`
4. Update `src/transport/mod.rs` to conditionally include the module
5. Update `src/manager.rs` to handle the new transport type
6. Add tests in the module
7. Add an example in `examples/`
8. Update documentation

### Adding a New Protocol

1. Create a new module in `src/protocol/`
2. Implement the `ProtocolHandler` trait
3. Add feature flag in `Cargo.toml`
4. Update `src/protocol/mod.rs` to conditionally include the module
5. Add tests in `tests/protocol_tests.rs`
6. Add an example in `examples/`
7. Update documentation

## Conditional Compilation

We use feature flags extensively. When adding platform-specific code:

```rust
// For a specific OS
#[cfg(target_os = "linux")]
fn linux_specific() { }

#[cfg(target_os = "windows")]
fn windows_specific() { }

#[cfg(target_os = "macos")]
fn macos_specific() { }

// For a feature flag
#[cfg(feature = "mqtt")]
mod mqtt {
    // MQTT-specific code
}

// Combining conditions
#[cfg(all(unix, feature = "serial"))]
fn unix_serial() { }
```

## Testing Platform-Specific Code

### Local Testing

Test on your platform:

```bash
# Test all features available on your platform
cargo test --all-features
```

### CI Testing

Our CI tests on:
- Linux (Ubuntu latest)
- macOS (latest)
- Windows (latest)

For both stable and beta Rust channels.

## Pull Request Process

1. **Fork and Clone**
   ```bash
   git clone https://github.com/your-username/rusty-s4i-io.git
   cd rusty-s4i-io
   ```

2. **Create a Feature Branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make Your Changes**
   - Write code following the existing style
   - Add tests for new functionality
   - Update documentation as needed
   - Run `cargo fmt` and `cargo clippy`

4. **Test Your Changes**
   ```bash
   # Run tests
   cargo test
   
   # Check formatting
   cargo fmt -- --check
   
   # Run clippy
   cargo clippy -- -D warnings
   
   # Build documentation
   cargo doc
   ```

5. **Commit Your Changes**
   ```bash
   git add .
   git commit -m "feat: add your feature description"
   ```

   We follow conventional commits:
   - `feat:` for new features
   - `fix:` for bug fixes
   - `docs:` for documentation changes
   - `test:` for test changes
   - `refactor:` for code refactoring
   - `chore:` for maintenance tasks

6. **Push and Create PR**
   ```bash
   git push origin feature/your-feature-name
   ```
   Then create a pull request on GitHub.

7. **Code Review**
   - Address any feedback from reviewers
   - Keep your PR up to date with main
   - Ensure CI passes

## Common Issues

### Build Failures

**Issue:** `libudev` not found (Linux)
```bash
sudo apt-get install libudev-dev
```

**Issue:** `libdbus` not found (Linux, for BLE)
```bash
sudo apt-get install libdbus-1-dev
```

**Issue:** USB feature won't build
```bash
# Linux
sudo apt-get install libudev-dev

# This feature may have additional requirements on your platform
```

### Test Failures

If tests fail on hardware-dependent features (serial, USB, BLE), it's expected if you don't have the hardware. These tests are primarily run in CI environments with mock hardware.

## Code of Conduct

- Be respectful and inclusive
- Welcome newcomers
- Focus on what is best for the community
- Show empathy towards other community members

## Questions?

- Open an issue for bugs or feature requests
- Check existing issues first
- For questions, consider opening a discussion

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
