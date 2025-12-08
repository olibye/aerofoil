# Feature Simplification Design

## Context
The project originally supported two Aeron client implementations via feature flags:
- **rusteron**: C++ bindings via FFI, mature, production-ready
- **aeron-rs**: Pure Rust implementation, no native dependencies

During implementation, aeron-rs was found to be incompatible with the rusteron-media-driver due to ring buffer capacity validation differences, so aeron-rs support was removed entirely.

Additionally, since there's only one backend now, the `rusteron` feature was removed and rusteron-client became a regular dependency.

## Goals
- Simplify dependency structure - rusteron is always available
- Separate the embedded media driver into its own feature
- Add support for external media drivers

## Non-Goals
- aeron-rs support (removed due to compatibility issues)

## Decisions

### Decision: Make rusteron-client a regular dependency
Removed the `rusteron` feature flag since there's only one Aeron client implementation.

**Rationale**: With only one backend, the feature gate adds unnecessary complexity. Users always get rusteron-client.

### Decision: Separate embedded-driver feature
Moved `rusteron-media-driver` to a separate `embedded-driver` feature.

**Rationale**: The media driver is only used in tests and benchmarks. The library itself only needs the client to connect to an externally running driver. This separation:
- Reduces dependency footprint for library users
- Makes the distinction clear: client vs embedded driver
- Allows users who run their own media driver to avoid compiling the C++ driver bindings

### Decision: Add external-driver feature
Added `external-driver` feature for users who want to run with an external media driver (Java or C++ aeronmd).

## Final Feature Set

```toml
[dependencies]
rusteron-client = "0.1"
rusteron-media-driver = { version = "0.1", optional = true }
wingfoil = "0.1"

[features]
default = []
embedded-driver = ["rusteron-media-driver"]
external-driver = []
dhat-heap = []
```

## Usage
- Library: `cargo build`
- Benchmarks with embedded driver: `cargo bench --features embedded-driver`
- Benchmarks with external driver: `cargo bench --features external-driver`
