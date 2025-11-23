# Integration Testing Strategy

This document describes how to run integration tests for aerofoil transport adapters that require external dependencies.

## Overview

Aerofoil has two types of tests:

1. **Unit tests**: Test code in isolation using manual test implementations (no external dependencies)
2. **Integration tests**: Test real Aeron transports requiring a media driver

## Prerequisites

### Aeron Media Driver

All integration tests require a running Aeron Media Driver.

#### Option 1: Download Pre-built Media Driver

1. Download from https://github.com/real-logic/aeron/releases
2. Extract the archive
3. Run the media driver:
   ```bash
   ./aeronmd
   ```

#### Option 2: Build from Source (macOS)

```bash
# Install dependencies
brew install cmake

# Clone and build
git clone https://github.com/real-logic/aeron.git
cd aeron

# Build using gradlew
./gradlew

# Run the media driver
./cppbuild/Release/binaries/aeronmd
```

**Note**: Building from source on macOS requires Xcode Command Line Tools:
```bash
xcode-select --install
```

#### Option 3: Using Homebrew (macOS)

```bash
# Install Aeron via Homebrew (if available)
brew tap real-logic/aeron
brew install aeron

# Run the media driver
aeronmd
```

**Note**: Homebrew tap availability may vary. Check https://github.com/real-logic/aeron for latest installation options.

#### Option 4: Embedded Media Driver (Advanced)

Some tests may use an embedded media driver that runs in-process. See specific test documentation.

### C++ Toolchain (Rusteron Tests Only)

Rusteron integration tests require a C++ toolchain on macOS:

```bash
# Install Xcode Command Line Tools
xcode-select --install

# Verify installation
clang++ --version
cmake --version
```

**Requirements**:
- C++17 compatible compiler (Clang from Xcode)
- CMake (install via `brew install cmake`)
- Standard build tools (included with Xcode Command Line Tools)

See [project.md](project.md#external-dependencies) for complete details.

## Running Integration Tests

### All Integration Tests

Integration tests are marked with `#[ignore]` by default to prevent them from running without a media driver.

```bash
# Start media driver first in a separate terminal
./aeronmd

# Run integration tests
cargo test --features rusteron -- --ignored --test-threads=1
```

**Note**: `--test-threads=1` prevents tests from interfering with each other when using shared channels.

### Rusteron Integration Tests

```bash
# With media driver running:
cargo test --features rusteron --test rusteron_integration -- --ignored
```

### Aeron-rs Integration Tests

```bash
# With media driver running:
cargo test --features aeron-rs --test aeron_rs_integration -- --ignored
```

## Writing Integration Tests

### File Structure

```
tests/
├── rusteron_integration.rs    # Rusteron-specific integration tests
├── aeron_rs_integration.rs    # Aeron-rs-specific integration tests
└── common/
    └── mod.rs                  # Shared test utilities
```

### Test Attributes

Mark integration tests with `#[ignore]` so they don't run by default:

```rust
#[test]
#[ignore]
fn test_rusteron_publish_subscribe() {
    // Given: media driver is running
    let channel = "aeron:ipc"; // Use IPC for tests
    let stream_id = 1001;

    // When: publish and subscribe
    // ... test implementation

    // Then: verify messages received
}
```

### Test Isolation

Each test should use unique channel/stream combinations:

```rust
// Good: unique stream ID per test
#[test]
#[ignore]
fn test_publication() {
    let stream_id = 1001;
    // ...
}

#[test]
#[ignore]
fn test_subscription() {
    let stream_id = 1002;  // Different from above
    // ...
}
```

### Cleanup

Always clean up resources in tests:

```rust
#[test]
#[ignore]
fn test_example() {
    let publisher = RusteronPublisher::new("aeron:ipc", 1001).unwrap();

    // Test code...

    // Cleanup happens automatically on drop, but can be explicit:
    drop(publisher);
}
```

## Continuous Integration

### GitHub Actions Example (macOS)

```yaml
name: Integration Tests

on: [push, pull_request]

jobs:
  integration-tests:
    runs-on: macos-latest

    steps:
      - uses: actions/checkout@v3

      - name: Install dependencies
        run: |
          brew install cmake

      - name: Install Aeron Media Driver
        run: |
          wget https://github.com/real-logic/aeron/releases/download/1.44.1/aeron-all-1.44.1.zip
          unzip aeron-all-1.44.1.zip
          chmod +x aeron-all-1.44.1/bin/aeronmd

      - name: Start Media Driver
        run: |
          aeron-all-1.44.1/bin/aeronmd &
          sleep 3  # Wait for driver to start

      - name: Run Integration Tests
        run: cargo test --features rusteron -- --ignored --test-threads=1

      - name: Stop Media Driver
        if: always()
        run: pkill aeronmd || true
```

**Note**: Adjust the Aeron version (1.44.1) to the latest release available.

## Troubleshooting

### Media Driver Not Running

**Error**: `TransportError::NotConnected` or similar

**Solution**: Start the media driver before running tests:
```bash
./aeronmd
```

### Port Conflicts

**Error**: Media driver fails to start with "address already in use"

**Solution**:
- Check if another media driver is running: `ps aux | grep aeronmd`
- Kill existing process: `pkill aeronmd`
- Or configure different ports in media driver properties

### C++ Compilation Errors (Rusteron)

**Error**: Build fails with C++ compiler errors

**Solution**:
- Ensure C++ toolchain is installed (see [project.md](project.md#external-dependencies))
- Check rusteron version compatibility
- Try: `cargo clean` then rebuild

### Test Timeouts

**Error**: Tests hang or timeout

**Possible causes**:
- Media driver not running
- Channel/stream ID conflicts between concurrent tests
- Resource leaks in test code

**Solutions**:
- Verify media driver is running: `ps aux | grep aeronmd`
- Use `--test-threads=1` to run tests sequentially
- Add explicit cleanup/drop calls
- Check for panics preventing cleanup

### Channel Configuration Issues

**Error**: `TransportError::InvalidChannel`

**Solution**:
- Verify channel URI format: `aeron:ipc` or `aeron:udp?endpoint=localhost:40123`
- Check media driver configuration matches channel requirements
- Ensure stream IDs are positive integers

## Performance Considerations

### Test Performance

Integration tests are slower than unit tests due to:
- Media driver communication overhead
- Network/IPC setup time
- Resource cleanup

**Best practices**:
- Use IPC channels (`aeron:ipc`) for faster tests
- Minimize number of integration tests (cover critical paths only)
- Use unit tests with manual implementations for most testing
- Run integration tests in CI only, not during development iteration

### Resource Limits

Media driver has resource limits:
- Maximum concurrent publications/subscriptions
- Buffer sizes
- Memory usage

Tests should be mindful of these limits and clean up properly.

## Examples

### Simple Publish/Subscribe Test

```rust
#[cfg(test)]
#[cfg(feature = "rusteron")]
mod integration_tests {
    use aerofoil::transport::{AeronPublisher, AeronSubscriber};
    use aerofoil::transport::rusteron::{RusteronPublisher, RusteronSubscriber};

    #[test]
    #[ignore]
    fn test_publish_and_receive() {
        // Given: Media driver is running
        let channel = "aeron:ipc";
        let stream_id = 1001;

        let mut publisher = RusteronPublisher::new(channel, stream_id)
            .expect("Failed to create publisher");
        let mut subscriber = RusteronSubscriber::new(channel, stream_id)
            .expect("Failed to create subscriber");

        // Wait for connection
        std::thread::sleep(std::time::Duration::from_millis(100));

        // When: Publish a message
        publisher.offer(b"Hello, Aeron!")
            .expect("Failed to publish");

        // Then: Receive the message
        let mut received = Vec::new();
        subscriber.poll(|fragment| {
            received.extend_from_slice(fragment.as_ref());
            Ok(())
        }).expect("Failed to poll");

        assert_eq!(received, b"Hello, Aeron!");
    }
}
```

### Error Handling Test

```rust
#[test]
#[ignore]
fn test_back_pressure() {
    // Given: Publisher with small buffer
    let mut publisher = RusteronPublisher::with_config(
        "aeron:ipc",
        1002,
        Config { buffer_size: 1024 }
    ).unwrap();

    // When: Flood with messages
    let mut back_pressure_count = 0;
    for _ in 0..10000 {
        match publisher.offer(&[0u8; 512]) {
            Ok(_) => {},
            Err(TransportError::BackPressure) => {
                back_pressure_count += 1;
            }
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }

    // Then: Some back-pressure should occur
    assert!(back_pressure_count > 0, "Expected back-pressure but got none");
}
```

## References

- Aeron documentation: https://github.com/real-logic/aeron/wiki
- Rusteron documentation: https://docs.rs/rusteron-client
- Aeron-rs documentation: https://docs.rs/aeron-rs
- Project testing strategy: [project.md](project.md#testing-strategy)
- Mocking guidelines: [mocking.md](mocking.md)
