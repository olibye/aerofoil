# Aerofoil

Wingfoil Aeron adapters for high-frequency trading.

## Overview

Aerofoil provides transport abstractions and adapters for integrating [Aeron](https://aeron.io/) messaging with [Wingfoil](https://crates.io/crates/wingfoil) stream processing.

## Backend Selection

Two Aeron backends are available:

| Aspect | rusteron (default) | aeron-rs |
|--------|-------------------|----------|
| Implementation | C++ wrapper (FFI) | Pure Rust |
| Maturity | More mature | Less mature |
| C++ toolchain | Required | Not required |
| Cross-compilation | Complex | Simpler |

### Using rusteron (default)

Requires C++ toolchain and Aeron C libraries:

```toml
[dependencies]
aerofoil = "0.1"
```

### Using aeron-rs (pure Rust)

No C++ toolchain required:

```toml
[dependencies]
aerofoil = { version = "0.1", default-features = false, features = ["aeron-rs"] }
```

## Usage

Both backends implement the same traits:

```rust
use aerofoil::transport::AeronPublisher;

fn publish<P: AeronPublisher>(publisher: &mut P, data: &[u8]) {
    publisher.offer(data).expect("publish failed");
}

// Use offer_mut to avoid copies on aeron-rs:
fn publish_mut<P: AeronPublisher>(publisher: &mut P, data: &mut [u8]) {
    publisher.offer_mut(data).expect("publish failed");
}
```

## Running Examples

Examples require the Aeron media driver. See `openspec/integration-test.md` for setup.

```bash
# With rusteron (default)
cargo run --example subscriber_node_value_access

# With aeron-rs
cargo run --example subscriber_node_value_access --no-default-features --features aeron-rs
```

## Development

```bash
# Build with rusteron (default)
cargo build

# Build with aeron-rs
cargo build --no-default-features --features aeron-rs

# Run tests
cargo test --lib
```

## Documentation

- `cargo doc --open` - API documentation
- `openspec/project.md` - Project conventions
- `openspec/integration-test.md` - Media driver setup

## Architecture

```
src/
├── transport/
│   ├── mod.rs         # AeronPublisher, AeronSubscriber traits
│   ├── rusteron/      # Rusteron adapter
│   └── aeron_rs/      # aeron-rs adapter
└── nodes/             # Wingfoil node implementations
```

## License

See LICENSE file.
