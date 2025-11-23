# Add Rusteron Adapter

## Why
Implement the transport traits for Rusteron, the wrapper around the official C++ Aeron client. Rusteron is mature, battle-tested, and provides proven performance for high-frequency trading systems. This change delivers the first working Aeron backend for aerofoil, enabling real message publication and subscription while validating that the trait design from `add-transport-traits` works with an actual Aeron implementation.

**Aligns with project conventions:**
- **Rusteron support**: Uses rusteron aeron client wrapper per project tech stack
- **Static dispatch**: Implements traits with zero runtime overhead
- **Zero-copy message handling**: Uses Rusteron's buffer claim APIs where possible
- **Non-blocking operations**: All implementations return immediately (critical for HFT)
- **Document latency compromises**: Per project conventions, document any cases where Rusteron requires copies
- **Document clone/copy cases**: Add comments explaining any clone or copy operations per project conventions

## What Changes
- Add Rusteron dependency with optional `rusteron` feature flag
- Implement `AeronPublisher` trait for Rusteron publication (offer method working, try_claim has TODO)
- Implement `AeronSubscriber` trait for Rusteron subscription (poll method working)
- Map Rusteron errors to common `TransportError` type
- Document latency compromises or clone/copy cases with explanations
- Add build script to detect Aeron Media Driver installation
- Create media driver helper module for integration tests
- Add unit tests verifying trait implementation (using manual test doubles per testing strategy)
- Create comprehensive README documenting status and requirements
- Use standard Rust formatting with `rustfmt`

## Impact
- Affected specs: `rusteron-adapter` (new capability)
- Affected code: Creates `src/transport/rusteron/` module, adds `build.rs`, creates `README.md`
- Dependencies: Adds `rusteron-client` as optional dependency (requires C++ toolchain)
- Builds on: `add-transport-traits` (requires traits to be defined first)
- User value: Functional Aeron publisher (offer) and subscriber (poll) for production HFT systems
- Testing: Unit tests with manual test implementations per project testing strategy
- Build infrastructure: Detects media driver, provides helper for integration tests
- Documentation: Comprehensive README with status, requirements, and quick start
- Status: **Partial implementation** - 39/44 tasks complete (89%)
