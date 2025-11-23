# Implementation Tasks

## 1. Core Trait Definitions
- [ ] 1.1 Create `src/transport/mod.rs` module structure
- [ ] 1.2 Define `AeronPublisher` trait with `offer` and `try_claim` methods
- [ ] 1.3 Define `AeronSubscriber` trait with `poll` method
- [ ] 1.4 Add trait documentation with usage examples

## 2. Error Types
- [ ] 2.1 Create `TransportError` enum in `src/transport/error.rs`
- [ ] 2.2 Add error variants: BackPressure, NotConnected, InvalidChannel, etc.
- [ ] 2.3 Implement `std::error::Error` and `Display` for `TransportError`
- [ ] 2.4 Add `source()` support for wrapping backend-specific errors

## 3. Buffer Abstractions
- [ ] 3.1 Define `ClaimBuffer<'a>` type for publication in `src/transport/buffer.rs`
- [ ] 3.2 Define `FragmentBuffer<'a>` type for subscription
- [ ] 3.3 Implement `Deref` and `DerefMut` for ergonomic buffer access
- [ ] 3.4 Add safety documentation for lifetime constraints

## 4. Mock Implementations
- [ ] 4.1 Create `src/transport/mock.rs` module
- [ ] 4.2 Implement `MockPublisher` with in-memory message queue
- [ ] 4.3 Implement `MockSubscriber` with controllable message injection
- [ ] 4.4 Add methods to inspect published messages and inject received messages
- [ ] 4.5 Add mockall `#[automock]` attribute to traits

## 5. Testing
- [ ] 5.1 Write unit tests for mock publisher behavior
- [ ] 5.2 Write unit tests for mock subscriber behavior
- [ ] 5.3 Write tests demonstrating generic code using trait bounds
- [ ] 5.4 Add documentation tests showing trait usage patterns
- [ ] 5.5 Test error handling and propagation

## 6. Documentation
- [ ] 6.1 Document `AeronPublisher` trait with rustdoc
- [ ] 6.2 Document `AeronSubscriber` trait with rustdoc
- [ ] 6.3 Add crate-level documentation explaining trait-based design
- [ ] 6.4 Document mock testing patterns and examples
- [ ] 6.5 Add examples showing zero-copy buffer usage
