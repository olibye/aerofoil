
# Add Transport Traits

## Why
Establish the foundational trait-based abstraction for Aeron transport operations that will enable zero-cost polymorphism across different backend implementations. This change provides the API contract that all adapters will implement, along with test implementations for testing. By defining traits first, we can validate the API design without committing to specific Aeron client dependencies.

## What Changes
- Define `AeronPublisher` and `AeronSubscriber` traits with core transport operations
- Create `TransportError` enum for unified error handling
- Define buffer abstraction types (`ClaimBuffer<'a>`, `FragmentBuffer<'a>`) for zero-copy access
- Provide manual test implementations demonstrating trait usage (no mockall)
- Document all design decisions in code comments
- Document trait API and usage patterns with comprehensive rustdoc
- Use standard Rust formatting with `rustfmt`

## Impact
- Affected specs: `transport-traits` (new capability)
- Affected code: Creates `src/transport/` module structure
- Dependencies: No new dependencies (manual test implementations, no mockall)
- Foundation for: Later changes will implement these traits for Rusteron and aeron-rs
- User value: Enables testing transport-dependent code without Aeron infrastructure
- Testing: Manual implementations in `src/transport/tests.rs` per project testing strategy
