# Implementation Tasks

## 1. Cargo.toml Feature Reorganization
- [ ] 1.1 Remove `rusteron-media-driver` from `rusteron` feature
- [ ] 1.2 Remove `rusteron-media-driver` from `aeron-rs` feature
- [ ] 1.3 Add `embedded-driver` feature that enables `rusteron-media-driver`
- [ ] 1.4 Verify `cargo build --features rusteron` compiles without media driver
- [ ] 1.5 Verify `cargo build --features aeron-rs` compiles without media driver

## 2. Source Code cfg Gate Updates
- [ ] 2.1 Update `src/transport/aeron_rs/mod.rs` to use `#[cfg(feature = "aeron-rs")]` instead of `#[cfg(all(feature = "aeron-rs", not(feature = "rusteron")))]`
- [ ] 2.2 Update any other source files with mutual exclusivity cfg guards
- [ ] 2.3 Verify `cargo build --features rusteron,aeron-rs` compiles successfully

## 3. Test/Benchmark Common Module Updates
- [ ] 3.1 Update `benches/common/mod.rs` MediaDriverGuard to use `#[cfg(feature = "embedded-driver")]`
- [ ] 3.2 Update `benches/common/mod.rs` aeron_rs_support to use `#[cfg(feature = "aeron-rs")]` (remove `not(feature = "rusteron")`)
- [ ] 3.3 Update `tests/common/mod.rs` MediaDriverGuard to use `#[cfg(feature = "embedded-driver")]`
- [ ] 3.4 Define stream ID ranges to avoid conflicts (rusteron: 1000-1999, aeron-rs: 2000-2999)

## 4. Publication Latency Benchmark Updates
- [ ] 4.1 Update `benches/publication_latency.rs` cfg guards to allow both backends
- [ ] 4.2 Update criterion_group to include both benchmark functions when both features enabled

## 5. Subscription Throughput Benchmark Updates
- [ ] 5.1 Update `benches/subscription_throughput.rs` similarly
- [ ] 5.2 Update criterion_group for both backends

## 6. Transceiver Benchmark Updates
- [ ] 6.1 Update `benches/transceiver.rs` similarly
- [ ] 6.2 Update criterion_group for both backends

## 7. Allocation Tracking Benchmark Updates
- [ ] 7.1 Update `benches/allocation_tracking.rs` similarly
- [ ] 7.2 Update criterion_group for both backends

## 8. Validation
- [ ] 8.1 Run `cargo test --features rusteron,aeron-rs,embedded-driver`
- [ ] 8.2 Run `cargo bench --features rusteron,aeron-rs,embedded-driver` and verify both backends execute
- [ ] 8.3 Verify Criterion generates comparison-friendly output
