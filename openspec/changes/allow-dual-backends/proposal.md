## Why
Currently, `rusteron` (C++ based) and `aeron-rs` (Pure Rust) backends are mutually exclusive. This makes it difficult to run side-by-side benchmarks to compare their performance in the same environment.

## What Changes
- Remove the mutual exclusivity check in `src/lib.rs`.
- Update `benches` to support running both backends when both features are enabled.
- Update `cargo` feature handling to allow coexistence.

## Impact
- Affected specs: `aeron-rs-adapter`
- Affected code: `src/lib.rs`, `benches/`
- Benchmarking: Enables `cargo bench --features "rusteron aeron-rs"` to run both sets of benchmarks.
