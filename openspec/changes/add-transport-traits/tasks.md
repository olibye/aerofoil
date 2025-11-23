# Implementation Tasks

## 1. Core Trait Definitions
- [x] 1.1 Create `src/transport/mod.rs` module structure
- [x] 1.2 Define `AeronPublisher` trait with `offer` and `try_claim` methods
- [x] 1.3 Define `AeronSubscriber` trait with `poll` method
- [x] 1.4 Add trait documentation with usage examples

## 2. Error Types
- [x] 2.1 Create `TransportError` enum in `src/transport/error.rs`
- [x] 2.2 Add error variants: BackPressure, NotConnected, InvalidChannel, etc.
- [x] 2.3 Implement `std::error::Error` and `Display` for `TransportError`
- [x] 2.4 Add `source()` support for wrapping backend-specific errors

## 3. Buffer Abstractions
- [x] 3.1 Define `ClaimBuffer<'a>` type for publication in `src/transport/buffer.rs`
- [x] 3.2 Define `FragmentBuffer<'a>` type for subscription
- [x] 3.3 Implement `Deref` and `DerefMut` for ergonomic buffer access
- [x] 3.4 Add safety documentation for lifetime constraints

## 4. Test Implementation Support
- [x] 4.1 Create `src/transport/tests.rs` with manual test implementations
- [x] 4.2 Implement `TestPublisher` demonstrating `AeronPublisher` trait
- [x] 4.3 Implement `TestSubscriber` demonstrating `AeronSubscriber` trait

## 5. Testing
- [x] 5.1 Write test for `TestPublisher::offer` method
- [x] 5.2 Write test for `TestPublisher::try_claim` method
- [x] 5.3 Write test for `TestSubscriber::poll` method
- [x] 5.4 Write test demonstrating generic code using trait bounds
- [x] 5.5 Test error handling and propagation with manual error implementations

## 6. Documentation
- [x] 6.1 Document `AeronPublisher` trait with rustdoc
- [x] 6.2 Document `AeronSubscriber` trait with rustdoc
- [x] 6.3 Add crate-level documentation explaining trait-based design
- [x] 6.4 Document manual test implementation patterns with examples
- [x] 6.5 Add examples showing zero-copy buffer usage
