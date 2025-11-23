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
- **C++ Toolchain** (for Rusteron):
  - Xcode Command Line Tools: `xcode-select --install`
  - CMake: `brew install cmake`

### For Running

- **Aeron Media Driver**: Must be running for Rusteron adapter to function
  - See [integration-test.md](openspec/integration-test.md) for setup instructions

## Quick Start

### Using Traits Only (No Aeron Required)

```rust
use aerofoil::transport::{AeronPublisher, TransportError};

// Implement the trait for testing
struct TestPublisher {
    messages: Vec<Vec<u8>>,
}

impl AeronPublisher for TestPublisher {
    fn offer(&mut self, buffer: &[u8]) -> Result<i64, TransportError> {
        self.messages.push(buffer.to_vec());
        Ok(self.messages.len() as i64 - 1)
    }

    fn try_claim<'a>(&'a mut self, _length: usize) -> Result<ClaimBuffer<'a>, TransportError> {
        todo!("Implement as needed for tests")
    }
}
```

### Using Rusteron (Requires Media Driver)

```rust
use aerofoil::transport::AeronPublisher;
use aerofoil::transport::rusteron::RusteronPublisher;
use rusteron_client::AeronPublication;

// Note: Requires media driver running and proper Rusteron setup
// This is a skeleton - actual initialization requires more boilerplate

fn example() -> Result<(), Box<dyn std::error::Error>> {
    // Create Rusteron publication (details omitted)
    // let publication = AeronPublication::new(...)?;

    // Wrap in our publisher
    // let mut publisher = RusteronPublisher::new(publication);

    // Publish message
    // let position = publisher.offer(b"Hello, Aeron!")?;

    Ok(())
}
```

## Documentation

- **[Project Conventions](openspec/project.md)** - Project standards and conventions
- **[Mocking Guidelines](openspec/mocking.md)** - When to use mockall vs manual implementations
- **[Integration Testing](openspec/integration-test.md)** - How to run integration tests with Aeron

## Architecture

```
aerofoil/
├── src/
│   ├── transport/
│   │   ├── mod.rs           # Trait definitions
│   │   ├── error.rs         # TransportError enum
│   │   ├── buffer.rs        # ClaimBuffer, FragmentBuffer
│   │   ├── tests.rs         # Manual test implementations
│   │   └── rusteron/        # Rusteron adapter (partial)
│   │       ├── mod.rs
│   │       ├── error.rs     # Error conversion
│   │       ├── publisher.rs # RusteronPublisher
│   │       └── subscriber.rs# RusteronSubscriber
│   └── lib.rs
├── openspec/                # OpenSpec change proposals
└── README.md
```

## Development

### Build

```bash
# With rusteron (default)
cargo build

# Without any backend
cargo build --no-default-features

# Run tests
cargo test
```

### Next Steps

See [openspec/changes/](openspec/changes/) for planned work:
1. ✅ `add-transport-traits` - Complete
2. 🚧 `add-rusteron-adapter` - In progress (offer/poll working, try_claim TODO)
3. ⏳ `add-aeron-rs-adapter` - Not started
4. ⏳ `add-transport-benchmarks` - Not started

## Contributing

This project follows the OpenSpec workflow. See [openspec/AGENTS.md](openspec/AGENTS.md) for details on proposing changes.

## License

[Add license information]
