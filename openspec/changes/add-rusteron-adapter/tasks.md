# Implementation Tasks

## 1. Dependency Setup
- [ ] 1.1 Add `rusteron-client` to `Cargo.toml` with `optional = true`
- [ ] 1.2 Create `rusteron` feature flag in `Cargo.toml`
- [ ] 1.3 Set `rusteron` as default feature
- [ ] 1.4 Document C++ toolchain requirements in README

## 2. Module Structure
- [ ] 2.1 Create `src/transport/rusteron/mod.rs` with conditional compilation
- [ ] 2.2 Add `pub mod rusteron` to `src/transport/mod.rs` with `#[cfg(feature = "rusteron")]`
- [ ] 2.3 Create `src/transport/rusteron/publisher.rs`
- [ ] 2.4 Create `src/transport/rusteron/subscriber.rs`
- [ ] 2.5 Create `src/transport/rusteron/error.rs` for error conversion

## 3. Publisher Implementation
- [ ] 3.1 Define `RusteronPublisher` struct wrapping Rusteron publication type
- [ ] 3.2 Implement `AeronPublisher::offer` using Rusteron's offer API
- [ ] 3.3 Implement `AeronPublisher::try_claim` using Rusteron's claim API (TODO: requires buffer access investigation)
- [ ] 3.4 Ensure proper lifetime handling for claimed buffers (blocked by 3.3)
- [ ] 3.5 Add constructor (`new`) and accessor (`inner`) methods

## 4. Subscriber Implementation
- [ ] 4.1 Define `RusteronSubscriber` struct wrapping Rusteron subscription type
- [ ] 4.2 Implement `AeronSubscriber::poll` using Rusteron's `poll_once` API
- [ ] 4.3 Handle fragment assembly (Rusteron handles this automatically)
- [ ] 4.4 Ensure proper lifetime handling for received buffers
- [ ] 4.5 Add constructor (`new`) and accessor (`inner`) methods

## 5. Error Handling
- [ ] 5.1 Implement conversion from Rusteron errors to `TransportError`
- [ ] 5.2 Map back-pressure conditions to `TransportError::BackPressure` (result code -2)
- [ ] 5.3 Map connection errors appropriately (result code -1, -4)
- [ ] 5.4 Preserve original Rusteron error as error string (via `Backend` variant)
- [ ] 5.5 Add error context for debugging (in error conversion functions)

## 6. Testing
- [ ] 6.1 Add compile-time tests verifying trait implementation
- [ ] 6.2 Write integration tests requiring Aeron media driver (deferred - requires media driver setup)
- [ ] 6.3 Test error handling and conversion (unit tests in error.rs)
- [ ] 6.4 Test zero-copy buffer access patterns (blocked by try_claim implementation)
- [ ] 6.5 Test non-blocking behavior (requires integration tests with media driver)

## 7. Examples
- [ ] 7.1 Create `examples/rusteron_publisher.rs` showing publication
- [ ] 7.2 Create `examples/rusteron_subscriber.rs` showing subscription
- [ ] 7.3 Add example README explaining how to run with media driver
- [ ] 7.4 Add example showing generic code using trait bounds

## 8. Documentation
- [ ] 8.1 Document `RusteronPublisher` with rustdoc (including design decisions)
- [ ] 8.2 Document `RusteronSubscriber` with rustdoc (including design decisions)
- [ ] 8.3 Document feature flag usage in crate-level docs (lib.rs)
- [ ] 8.4 Add documentation about C++ toolchain requirements (README.md, integration-test.md)
- [ ] 8.5 Document Rusteron usage patterns in module docs

## 9. Build Infrastructure
- [ ] 9.1 Create `build.rs` to detect Aeron media driver at compile time
- [ ] 9.2 Check for `aeronmd` in PATH and common macOS locations
- [ ] 9.3 Set `AERON_MEDIA_DRIVER_PATH` environment variable when found
- [ ] 9.4 Provide helpful warnings if media driver not found
- [ ] 9.5 Create `MediaDriverGuard` test helper for RAII driver management
- [ ] 9.6 Document build-time detection in README

## Status Summary

**Completed**: 0/44 tasks (0%)

**Functional**:
- ❌ All tasks rolled back to pending state

**Next Steps**:
1. Start fresh implementation from task 1.1
