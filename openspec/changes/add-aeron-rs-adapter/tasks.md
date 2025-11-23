# Implementation Tasks

## 1. Dependency Setup
- [ ] 1.1 Add `aeron-rs` to `Cargo.toml` with `optional = true` and `package = "aeron"`
- [ ] 1.2 Create `aeron-rs` feature flag in `Cargo.toml`
- [ ] 1.3 Add compile-time mutual exclusivity check with Rusteron
- [ ] 1.4 Document pure Rust deployment option in README

## 2. Module Structure
- [ ] 2.1 Create `src/transport/aeron_rs/mod.rs` with conditional compilation
- [ ] 2.2 Add `pub mod aeron_rs` to `src/transport/mod.rs` with `#[cfg(feature = "aeron-rs")]`
- [ ] 2.3 Create `src/transport/aeron_rs/publisher.rs`
- [ ] 2.4 Create `src/transport/aeron_rs/subscriber.rs`
- [ ] 2.5 Create `src/transport/aeron_rs/error.rs` for error conversion

## 3. Mutual Exclusivity Check
- [ ] 3.1 Add compile_error! macro in `src/lib.rs` checking both features
- [ ] 3.2 Test that enabling both features produces clear error message
- [ ] 3.3 Document feature flag usage in crate docs

## 4. Publisher Implementation
- [ ] 4.1 Define `AeronRsPublisher` struct wrapping aeron-rs publication type
- [ ] 4.2 Implement `AeronPublisher::offer` using aeron-rs offer API
- [ ] 4.3 Implement `AeronPublisher::try_claim` using aeron-rs claim API
- [ ] 4.4 Ensure proper lifetime handling for claimed buffers
- [ ] 4.5 Add constructor and configuration methods

## 5. Subscriber Implementation
- [ ] 5.1 Define `AeronRsSubscriber` struct wrapping aeron-rs subscription type
- [ ] 5.2 Implement `AeronSubscriber::poll` using aeron-rs poll API
- [ ] 5.3 Handle fragment assembly if needed
- [ ] 5.4 Ensure proper lifetime handling for received buffers
- [ ] 5.5 Add constructor and configuration methods

## 6. Error Handling
- [ ] 6.1 Implement conversion from aeron-rs errors to `TransportError`
- [ ] 6.2 Map back-pressure conditions to `TransportError::BackPressure`
- [ ] 6.3 Map connection errors appropriately
- [ ] 6.4 Preserve original aeron-rs error as error source
- [ ] 6.5 Add error context for debugging

## 7. Testing
- [ ] 7.1 Write unit tests using mock components
- [ ] 7.2 Write integration tests requiring Aeron media driver
- [ ] 7.3 Test error handling and conversion
- [ ] 7.4 Test zero-copy buffer access patterns
- [ ] 7.5 Test non-blocking behavior
- [ ] 7.6 Test mutual exclusivity compile error (expected failure test)

## 8. CI Matrix
- [ ] 8.1 Add CI job testing with `--features rusteron`
- [ ] 8.2 Add CI job testing with `--features aeron-rs`
- [ ] 8.3 Add CI job testing with `--no-default-features`
- [ ] 8.4 Verify both backends pass all tests

## 9. Examples
- [ ] 9.1 Create `examples/aeron_rs_publisher.rs` showing publication
- [ ] 9.2 Create `examples/aeron_rs_subscriber.rs` showing subscription
- [ ] 9.3 Add example README explaining how to run with media driver
- [ ] 9.4 Add example showing generic code that works with either backend

## 10. Documentation
- [ ] 10.1 Document `AeronRsPublisher` with rustdoc
- [ ] 10.2 Document `AeronRsSubscriber` with rustdoc
- [ ] 10.3 Document feature flag usage and backend selection in crate-level docs
- [ ] 10.4 Add comparison table: Rusteron vs aeron-rs (maturity, deployment, performance)
- [ ] 10.5 Document zero-copy usage patterns specific to aeron-rs
