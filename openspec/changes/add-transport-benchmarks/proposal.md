# Add Transport Benchmarks

## Why
With both Rusteron and aeron-rs adapters implemented, we need objective performance data to help users choose between backends. Benchmarks will validate the zero-cost abstraction claims, verify zero-copy behavior, and provide quantitative comparison of publication latency and subscription throughput. This enables data-driven backend selection and helps detect performance regressions in CI.

**Aligns with project conventions:**
- **Benchmarking strategy**: Uses Criterion per project tech stack
- **Combine examples into benchmarks**: Per project conventions, examples will serve as benchmark code
- **Zero-copy validation**: Verifies zero-copy message handling where possible
- **Static dispatch validation**: Confirms no runtime overhead from trait abstraction
- **Document latency compromises**: Benchmark results will document any performance differences between rusteron and aeron-rs

## What Changes
- Set up Criterion.rs benchmarking framework per project tech stack
- Create publication latency benchmarks for both Rusteron and aeron-rs
- Create subscription throughput benchmarks for both backends
- Create combined pub/sub (transceiver) benchmarks measuring concurrent publish and subscribe
- Add request/response roundtrip latency benchmarks
- Add allocation tracking to verify zero-copy behavior in hot path
- Combine examples into benchmarks where possible per project conventions
- Document benchmark methodology and how to interpret results
- Document latency compromises between rusteron and aeron-rs based on results
- Add CI job to run benchmarks and detect performance regressions
- Use standard Rust formatting with `rustfmt`

## Impact
- Affected specs: `transport-benchmarks` (new capability)
- Affected code: Creates `benches/` directory with benchmark suites
- Dependencies: Adds `criterion` as dev-dependency per project tech stack
- Builds on: Requires both `add-rusteron-adapter` and `add-aeron-rs-adapter`
- User value: Objective performance data for informed backend selection
- Documentation: Results will inform documentation of latency compromises per project conventions
