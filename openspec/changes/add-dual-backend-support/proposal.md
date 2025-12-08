# Add Dual Backend Support

## Why
Currently, rusteron and aeron-rs features are mutually exclusive (`#[cfg(all(feature = "aeron-rs", not(feature = "rusteron")))]`), requiring separate compilation and benchmark runs to compare them. This makes side-by-side performance comparisons cumbersome and prevents Criterion from generating unified comparison reports.

## What Changes
- **BREAKING**: Remove mutual exclusivity constraint between `rusteron` and `aeron-rs` features
- Update `#[cfg]` gates from `#[cfg(all(feature = "aeron-rs", not(feature = "rusteron")))]` to `#[cfg(feature = "aeron-rs")]`
- Move `rusteron-media-driver` from feature dependencies to separate `embedded-driver` feature (only needed for tests/benchmarks)
- Refactor benchmark `BenchContext` modules to support running both backends in a single benchmark run
- Update Criterion benchmark groups to enable unified comparison reports

## Impact
- Affected specs: `transport-traits`, `aeron-rs-adapter`
- Affected code:
  - `Cargo.toml` (feature reorganization, media-driver separation)
  - `src/transport/aeron_rs/mod.rs` (cfg gate simplification)
  - `benches/common/mod.rs` (cfg gate simplification, embedded-driver feature)
  - `tests/common/mod.rs` (embedded-driver feature)
  - `benches/publication_latency.rs` (unified benchmark groups)
  - `benches/subscription_throughput.rs` (unified benchmark groups)
  - `benches/transceiver.rs` (unified benchmark groups)
  - `benches/allocation_tracking.rs` (unified benchmark groups)
- User value:
  - `cargo bench --features rusteron,aeron-rs,embedded-driver` produces side-by-side comparisons
  - Lighter library for users who run their own media driver
