# Add Transport Benchmarks

## Why
With both Rusteron and aeron-rs adapters implemented, we need objective performance data to help users choose between backends. Benchmarks will validate the zero-cost abstraction claims, verify zero-copy behavior, and provide quantitative comparison of publication latency and subscription throughput. This enables data-driven backend selection and helps detect performance regressions in CI.

## What Changes
- Set up Criterion.rs benchmarking framework
- Create publication latency benchmarks for both Rusteron and aeron-rs
- Create subscription throughput benchmarks for both backends
- Add allocation tracking to verify zero-copy behavior in hot path
- Document benchmark methodology and how to interpret results
- Add CI job to run benchmarks and detect performance regressions

## Impact
- Affected specs: `transport-benchmarks` (new capability)
- Affected code: Creates `benches/` directory with benchmark suites
- Dependencies: Adds `criterion` as dev-dependency
- Builds on: Requires both `add-rusteron-adapter` and `add-aeron-rs-adapter`
- User value: Objective performance data for informed backend selection
