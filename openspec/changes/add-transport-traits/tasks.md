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

## 4. Mockall Integration
- [ ] 4.1 Add `#[cfg_attr(test, automock)]` attribute to `AeronPublisher` trait
- [ ] 4.2 Add `#[cfg_attr(test, automock)]` attribute to `AeronSubscriber` trait
- [ ] 4.3 Verify `MockAeronPublisher` and `MockAeronSubscriber` are generated in tests

## 5. Testing
- [ ] 5.1 Write tests using `MockAeronPublisher` with mockall expectations
- [ ] 5.2 Write tests using `MockAeronSubscriber` with mockall expectations
- [ ] 5.3 Write tests demonstrating generic code using trait bounds
- [ ] 5.4 Add documentation tests showing trait usage patterns
- [ ] 5.5 Test error handling and propagation with mocks

## 6. Documentation
- [ ] 6.1 Document `AeronPublisher` trait with rustdoc
- [ ] 6.2 Document `AeronSubscriber` trait with rustdoc
- [ ] 6.3 Add crate-level documentation explaining trait-based design
- [ ] 6.4 Document mockall testing patterns with examples
- [ ] 6.5 Add examples showing zero-copy buffer usage
