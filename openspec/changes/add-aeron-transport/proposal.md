# Add Aeron Transport Adapters

## Why
The project needs to support both Rusteron (C++ Aeron wrapper) and aeron-rs (pure Rust client) as transport options. Different deployment scenarios may favor one over the other - Rusteron offers maturity and proven performance, while aeron-rs provides pure Rust memory safety and simpler deployment. Creating zero-cost abstraction adapters allows application code to remain transport-agnostic while maintaining the low-latency, non-blocking requirements critical for high-frequency trading systems.
This abstraction support testing with mock implementations.

## What Changes
- Add trait-based abstractions for Aeron publication and subscription operations
- Implement zero-cost adapters for both Rusteron and aeron-rs clients
- Support compile-time transport selection via feature flags
- Ensure zero-copy message handling patterns work across both implementations
- Provide common error types that unify the different client error models
- Add bench marking of both implementations for comparison

## Impact
- Affected specs: `aeron-transport` (new capability)
- Affected code: `src/lib.rs` will be replaced with modular transport adapter implementation
- Dependencies: Will add conditional dependencies on `rusteron` and/or `aeron-rs` based on feature flags
- Performance: Zero runtime overhead - all abstractions resolved at compile time
