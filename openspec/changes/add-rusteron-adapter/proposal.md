# Add Rusteron Adapter

## Why
Implement the transport traits for Rusteron, the wrapper around the official C++ Aeron client. Rusteron is mature, battle-tested, and provides proven performance for high-frequency trading systems. This change delivers the first working Aeron backend for aerofoil, enabling real message publication and subscription while validating that the trait design from `add-transport-traits` works with an actual Aeron implementation.

## What Changes
- Add Rusteron dependency with optional `rusteron` feature flag
- Implement `AeronPublisher` trait for Rusteron publication
- Implement `AeronSubscriber` trait for Rusteron subscription
- Map Rusteron errors to common `TransportError` type
- Ensure zero-copy patterns using Rusteron's buffer access APIs
- Provide examples demonstrating Rusteron usage
- Add tests verifying trait implementation

## Impact
- Affected specs: `rusteron-adapter` (new capability)
- Affected code: Creates `src/transport/rusteron/` module
- Dependencies: Adds `rusteron` as optional dependency (requires C++ toolchain)
- Builds on: `add-transport-traits` (requires traits to be defined first)
- User value: Working Aeron publisher and subscriber for production HFT systems
