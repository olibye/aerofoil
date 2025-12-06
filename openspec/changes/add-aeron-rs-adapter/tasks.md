# Implementation Tasks

## 1. Dependency Setup
- [x] 1.1 Add `aeron-rs` to `Cargo.toml` with `optional = true`
- [x] 1.2 Create `aeron-rs` feature flag in `Cargo.toml`
- [x] 1.3 Add compile-time mutual exclusivity check with Rusteron
- [x] 1.4 Document pure Rust deployment option in README

## 2. Module Structure
- [x] 2.1 Create `src/transport/aeron_rs/mod.rs` with conditional compilation
- [x] 2.2 Add `pub mod aeron_rs` to `src/transport/mod.rs` with `#[cfg(feature = "aeron-rs")]`
- [x] 2.3 Create `src/transport/aeron_rs/publisher.rs`
- [x] 2.4 Create `src/transport/aeron_rs/subscriber.rs`
- [x] 2.5 Create `src/transport/aeron_rs/error.rs` for error conversion

## 3. Mutual Exclusivity Check
- [x] 3.1 Add compile_error! macro in `src/lib.rs` checking both features
- [x] 3.2 Test that enabling both features produces clear error message
- [x] 3.3 Document feature flag usage in crate docs

## 4. Publisher Implementation
- [x] 4.1 Define `AeronRsPublisher` struct wrapping aeron-rs publication type
- [x] 4.2 Implement `AeronPublisher::offer` using aeron-rs offer API
- [x] 4.3 Implement `AeronPublisher::offer_mut` for mutable buffer API
- [x] 4.4 Implement `AeronPublisher::try_claim` using aeron-rs claim API
- [x] 4.5 Add constructor method

## 5. Subscriber Implementation
- [x] 5.1 Define `AeronRsSubscriber` struct wrapping aeron-rs subscription type
- [x] 5.2 Implement `AeronSubscriber::poll` using aeron-rs poll API
- [x] 5.3 Handle fragment callback conversion
- [x] 5.4 Add constructor method

## 6. Error Handling
- [x] 6.1 Implement conversion from aeron-rs errors to `TransportError`
- [x] 6.2 Map back-pressure conditions to `TransportError::BackPressure`
- [x] 6.3 Map connection errors appropriately
- [x] 6.4 Add unit tests for error conversion

## 7. Testing
- [x] 7.1 Write unit tests for error conversion
- [ ] 7.2 Write integration tests requiring Aeron media driver (deferred - requires media driver setup)
- [x] 7.3 Test error handling and conversion
- [x] 7.4 Test mutual exclusivity compile error

## 8. CI Matrix
- [x] 8.1 Add CI job testing with `--features rusteron`
- [x] 8.2 Add CI job testing with `--features aeron-rs`
- [x] 8.3 Add CI job testing with `--no-default-features`
- [x] 8.4 Add format check job

## 9. Examples
- [ ] 9.1-9.4 Create aeron-rs examples (deferred - existing examples are rusteron-specific)

## 10. Documentation
- [x] 10.1 Document `AeronRsPublisher` with rustdoc
- [x] 10.2 Document `AeronRsSubscriber` with rustdoc
- [x] 10.3 Document feature flag usage and backend selection in crate-level docs
- [x] 10.4 Add comparison table: Rusteron vs aeron-rs
- [x] 10.5 Document `offer_mut` usage patterns
