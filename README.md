# Aerofoil

Wingfoil Aeron adapters for high-frequency trading.

## Overview

Aerofoil provides transport abstractions and adapters for integrating [Aeron](https://aeron.io/) messaging with [Wingfoil](https://crates.io/crates/wingfoil) stream processing.

## Backend

Aerofoil uses [rusteron](https://crates.io/crates/rusteron-client) — a Rust wrapper over the Aeron C++ client — as its sole transport backend. A C++ toolchain and the Aeron C libraries are required to build.

```toml
[dependencies]
aerofoil = "0.1"
```

### Why not aeron-rs?

A pure-Rust [aeron-rs](https://crates.io/crates/aeron-rs) backend was prototyped behind a Cargo feature so users could avoid the C++ toolchain, but it was removed because the integration and benchmark tests could not be made to pass against it reliably. Only rusteron is supported today; if a working pure-Rust backend becomes viable it may be revisited.

## Usage

```rust
use aerofoil::transport::AeronPublisher;

fn publish<P: AeronPublisher>(publisher: &mut P, data: &[u8]) {
    publisher.offer(data).expect("publish failed");
}
```

## Running Examples

Examples require the Aeron media driver. See `docs/development-guide.md` for setup.

```bash
cargo run --example subscriber_node_value_access
```

## Development

```bash
cargo build
cargo test --lib
```

## Documentation

- `cargo doc --open` - API documentation
- `docs/architecture.md` - Architecture and conventions
- `docs/development-guide.md` - Testing and media driver setup

## Architecture

```text
src/
├── transport/
│   ├── mod.rs         # AeronPublisher, AeronSubscriber traits
│   └── rusteron/      # Rusteron adapter
└── nodes/             # Wingfoil node implementations
```

## License

Licensed under the [MIT License](LICENSE).
