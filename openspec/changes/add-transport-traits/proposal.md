
# Add Transport Traits

## Why
Establish the foundational trait-based abstraction for Aeron transport operations that will enable zero-cost polymorphism across different backend implementations. This change provides the API contract that all adapters will implement, along with mock implementations for testing. By defining traits first, we can validate the API design without committing to specific Aeron client dependencies.

## What Changes
- Define `AeronPublisher` and `AeronSubscriber` traits with core transport operations
- Create `TransportError` enum for unified error handling
- Define buffer abstraction types for zero-copy access
- Design traits to be easily implementable for testing (no Aeron required)
- Document the trait API and usage patterns

## Impact
- Affected specs: `transport-traits` (new capability)
- Affected code: Creates `src/transport/` module structure
- Dependencies: No new dependencies (mockall removed - traits are simple enough to implement manually for tests)
- Foundation for: Later changes will implement these traits for Rusteron and aeron-rs
- User value: Enables testing transport-dependent code without Aeron infrastructure
