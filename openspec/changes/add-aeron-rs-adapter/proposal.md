# Add Aeron-rs Adapter

## Why
Implement the transport traits for aeron-rs, the pure Rust Aeron client implementation. While Rusteron provides maturity and proven performance, aeron-rs offers pure Rust memory safety, simpler deployment (no C++ toolchain), and potential for Rust-specific optimizations. This change provides deployment flexibility, allowing users to choose between mature C++ wrapper (Rusteron) and pure Rust implementation based on their requirements.

## What Changes
- Add aeron-rs dependency with optional `aeron-rs` feature flag
- Implement `AeronPublisher` trait for aeron-rs publication
- Implement `AeronSubscriber` trait for aeron-rs subscription
- Map aeron-rs errors to common `TransportError` type
- Ensure zero-copy patterns using aeron-rs buffer access APIs
- Add compile-time checks ensuring mutual exclusivity with Rusteron feature
- Add CI matrix testing both backend feature flags
- Provide examples demonstrating aeron-rs usage

## Impact
- Affected specs: `aeron-rs-adapter` (new capability)
- Affected code: Creates `src/transport/aeron_rs/` module
- Dependencies: Adds `aeron-rs` as optional dependency (pure Rust, no C++ required)
- Builds on: `add-transport-traits` (requires traits to be defined first)
- Parallel to: `add-rusteron-adapter` (alternative backend implementation)
- User value: Pure Rust deployment option without C++ toolchain dependency
