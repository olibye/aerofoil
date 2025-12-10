# Simplify Benchmarks

## Why
The benchmarks were originally designed to compare two Aeron backends (rusteron vs aeron-rs). Now that aeron-rs has been removed due to compatibility issues, the "bare vs aerofoil" comparisons and multi-backend structure are unnecessary complexity. The benchmarks should be simplified to only measure what aerofoil actually provides.

## What Changes
- Remove "bare" rusteron API benchmarks (the trait abstraction overhead is negligible)
- Remove `try_claim` benchmarks (not yet implemented, returns error)
- Simplify benchmark group naming (remove "/aerofoil" suffix)
- Keep meaningful benchmarks: offer, poll, poll_parse, poll_burst, transceiver scenarios, allocations
- Update documentation to reflect single-backend reality

## Impact
- Affected code: `benches/*.rs` files
- Removes redundant benchmarks that duplicate measurements
- Cleaner benchmark output with fewer groups
- Faster benchmark runs (fewer redundant tests)
