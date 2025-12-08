# Implementation Tasks

## 1. Cargo.toml Feature Reorganization
- [x] 1.1 Remove `rusteron-media-driver` from `rusteron` feature
- [x] 1.2 Remove `rusteron-media-driver` from `aeron-rs` feature
- [x] 1.3 Add `embedded-driver` feature that enables `rusteron-media-driver`
- [x] 1.4 Verify `cargo build --features rusteron` compiles without media driver
- [x] 1.5 Verify `cargo build --features aeron-rs` compiles without media driver

## 2. Source Code cfg Gate Updates
- [x] 2.1 Update `src/lib.rs` to remove compile_error for both features enabled
- [x] 2.2 Update documentation to reflect both features can be enabled together
- [x] 2.3 Verify `cargo build --features rusteron,aeron-rs` compiles successfully

## 3. Test/Benchmark Common Module Updates
- [x] 3.1 Update `benches/common/mod.rs` MediaDriverGuard to use `#[cfg(feature = "embedded-driver")]`
- [x] 3.2 Update `benches/common/mod.rs` aeron_rs_support to use `#[cfg(feature = "aeron-rs")]` (remove `not(feature = "rusteron")`)
- [x] 3.3 Update `tests/common/mod.rs` MediaDriverGuard to use `#[cfg(feature = "embedded-driver")]`
- [x] 3.4 Define stream ID ranges to avoid conflicts (rusteron: 1000-14999, aeron-rs: 17000-23999)

## 4. Publication Latency Benchmark Updates
- [x] 4.1 Update `benches/publication_latency.rs` cfg guards to allow both backends
- [x] 4.2 Update criterion_group to include both benchmark functions when both features enabled

## 5. Subscription Throughput Benchmark Updates
- [x] 5.1 Update `benches/subscription_throughput.rs` similarly
- [x] 5.2 Update criterion_group for both backends

## 6. Transceiver Benchmark Updates
- [x] 6.1 Update `benches/transceiver.rs` similarly
- [x] 6.2 Update criterion_group for both backends

## 7. Allocation Tracking Benchmark Updates
- [x] 7.1 Update `benches/allocation_tracking.rs` similarly
- [x] 7.2 Update criterion_group for both backends

## 8. Validation
- [x] 8.1 Run `cargo build --features rusteron,aeron-rs` - compiles successfully
- [x] 8.2 Run `cargo test --features rusteron,aeron-rs` - compiles successfully
- [x] 8.3 Run `cargo build --benches --features rusteron,aeron-rs,embedded-driver` - compiles successfully
