# Add Aeron-rs Adapter

## Why
Implement the transport traits for aeron-rs, the pure Rust Aeron client implementation. While Rusteron provides maturity and proven performance, aeron-rs offers pure Rust memory safety, simpler deployment (no C++ toolchain), and potential for Rust-specific optimizations. This change provides deployment flexibility, allowing users to choose between mature C++ wrapper (Rusteron) and pure Rust implementation based on their requirements.

**Aligns with project conventions:**
- **Aeron-rs support**: Uses pure rust aeron client per project tech stack
- **Feature flag support**: Implements `aeron-rs` feature, mutually exclusive with `rusteron`
- **Static dispatch**: Implements traits with zero runtime overhead
- **Zero-copy message handling**: Uses aeron-rs buffer APIs where possible
- **Non-blocking operations**: All implementations return immediately (critical for HFT)
- **Document latency compromises**: Per project conventions, document differences from Rusteron
- **Document clone/copy cases**: Add comments explaining any clone or copy operations per project conventions

## What Changes
- Add aeron-rs dependency with optional `aeron-rs` feature flag
- Implement `AeronPublisher` trait for aeron-rs publication
- Implement `AeronSubscriber` trait for aeron-rs subscription
- Map aeron-rs errors to common `TransportError` type
- Ensure zero-copy patterns using aeron-rs buffer access APIs
- Document latency compromises between rusteron and aeron-rs per project conventions
- Document any clone/copy cases with explanations
- Add compile-time checks ensuring mutual exclusivity with Rusteron feature
- Add CI matrix testing both backend feature flags
- Provide examples demonstrating aeron-rs usage
- Add unit tests verifying trait implementation (using manual test doubles per testing strategy)
- Use standard Rust formatting with `rustfmt`

## Impact
- Affected specs: `aeron-rs-adapter` (new capability)
- Affected code: Creates `src/transport/aeron_rs/` module
- Dependencies: Adds `aeron-rs` as optional dependency (pure Rust, no C++ required)
- Builds on: `add-transport-traits` (requires traits to be defined first)
- Parallel to: `add-rusteron-adapter` (alternative backend implementation)
- User value: Pure Rust deployment option without C++ toolchain dependency
- Testing: Unit tests with manual test implementations per project testing strategy
