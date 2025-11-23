# Aerofoil

Rust library providing Wingfoil Aeron transport adapters with zero-cost abstraction.

## Status

**Current Implementation**: Partial Rusteron adapter

✅ **Completed**:
- Core transport traits (`AeronPublisher`, `AeronSubscriber`)
- Error handling (`TransportError`)
- Zero-copy buffer types (`ClaimBuffer`, `FragmentBuffer`)
- Manual test implementation patterns
- Rusteron integration:
  - `RusteronPublisher::offer()` - ✅ Working
  - `RusteronSubscriber::poll()` - ✅ Working
  - Error conversion from Rusteron to `TransportError`

⚠️ **Known Limitations**:
- `RusteronPublisher::try_claim()` - Not yet implemented (has TODO)
  - Requires investigation of Rusteron's `AeronBufferClaim` API
  - Needs to expose mutable buffer access with correct lifetime bounds
- No integration tests (requires Aeron Media Driver setup)
- No examples yet

🚧 **Not Yet Implemented**:
- Aeron-rs adapter
- Transport benchmarks
- Integration tests with media driver

## Requirements

### For Development

- **Rust**: Edition 2021
- **C++ Toolchain** (for Rusteron): See [integration-test.md](openspec/integration-test.md)

### For Running Integration Tests

- **Aeron Media Driver**: See [integration-test.md](openspec/integration-test.md) for installation

The build script checks for aeronmd and provides warnings if not found.

## Quick Start

See working examples in:
- `examples/` directory - Runnable examples demonstrating usage
- `#[cfg(test)] mod tests` blocks in source - Test patterns
- API documentation: `cargo doc --open`

## Documentation

- **[Project Conventions](openspec/project.md)** - Project standards and conventions
- **[Mocking Guidelines](openspec/mocking.md)** - When to use mockall vs manual implementations
- **[Integration Testing](openspec/integration-test.md)** - Installing dependencies for integration tests

## Architecture

```
aerofoil/
├── src/
│   ├── transport/
│   │   ├── mod.rs           # Trait definitions (tests inline)
│   │   ├── error.rs         # TransportError enum (tests inline)
│   │   ├── buffer.rs        # ClaimBuffer, FragmentBuffer (tests inline)
│   │   └── rusteron/        # Rusteron adapter (partial)
│   │       ├── mod.rs
│   │       ├── error.rs     # Error conversion
│   │       ├── publisher.rs # RusteronPublisher
│   │       └── subscriber.rs# RusteronSubscriber
│   └── lib.rs
├── tests/                   # Integration tests
├── examples/                # Runnable examples
├── openspec/                # OpenSpec change proposals
└── README.md
```

## Development

**Build**: `cargo build`

**Test**: `cargo test`

**With specific features**:
- Rusteron (default): `cargo build --features rusteron`
- No backend: `cargo build --no-default-features`

## Next Steps

See [openspec/changes/](openspec/changes/) for planned work:
1. ✅ `add-transport-traits` - Complete
2. 🚧 `add-rusteron-adapter` - In progress (offer/poll working, try_claim TODO)
3. ⏳ `add-aeron-rs-adapter` - Not started
4. ⏳ `add-transport-benchmarks` - Not started

## Contributing

This project follows the OpenSpec workflow. See [openspec/AGENTS.md](openspec/AGENTS.md) for details on proposing changes.

## License

[Add license information]
