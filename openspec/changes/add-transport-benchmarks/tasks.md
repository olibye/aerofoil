# Implementation Tasks

## 1. Benchmark Framework Setup
- [ ] 1.1 Add `criterion` to `[dev-dependencies]` in Cargo.toml
- [ ] 1.2 Add `dhat` or similar allocation profiler to dev-dependencies
- [ ] 1.3 Create `benches/` directory
- [ ] 1.4 Configure `[[bench]]` entries in Cargo.toml

## 2. Publication Latency Benchmarks
- [ ] 2.1 Create `benches/publication_latency.rs`
- [ ] 2.2 Write benchmark for Rusteron publication latency (offer method)
- [ ] 2.3 Write benchmark for aeron-rs publication latency (offer method)
- [ ] 2.4 Write benchmark for Rusteron zero-copy publication (try_claim)
- [ ] 2.5 Write benchmark for aeron-rs zero-copy publication (try_claim)
- [ ] 2.6 Add different message size scenarios (small, medium, large)

## 3. Subscription Throughput Benchmarks
- [ ] 3.1 Create `benches/subscription_throughput.rs`
- [ ] 3.2 Write benchmark for Rusteron subscription throughput
- [ ] 3.3 Write benchmark for aeron-rs subscription throughput
- [ ] 3.4 Test with different message rates (low, medium, high)
- [ ] 3.5 Measure messages per second and latency percentiles (p50, p99, p999)

## 4. Allocation Tracking
- [ ] 4.1 Create `benches/allocation_tracking.rs`
- [ ] 4.2 Set up allocation profiler integration
- [ ] 4.3 Verify zero allocations in Rusteron publication hot path
- [ ] 4.4 Verify zero allocations in aeron-rs publication hot path
- [ ] 4.5 Verify zero allocations in Rusteron subscription hot path
- [ ] 4.6 Verify zero allocations in aeron-rs subscription hot path
- [ ] 4.7 Document any unavoidable allocations and their causes

## 5. Benchmark Configuration
- [ ] 5.1 Configure Criterion settings (warm-up, measurement time, sample size)
- [ ] 5.2 Set up feature-gated benchmarks for each backend
- [ ] 5.3 Create benchmark helper functions to reduce duplication
- [ ] 5.4 Add command-line options for selective benchmark execution

## 6. Comparison Tooling
- [ ] 6.1 Create script to run benchmarks for both backends sequentially
- [ ] 6.2 Parse Criterion output and generate comparison report
- [ ] 6.3 Create markdown table or chart comparing results
- [ ] 6.4 Add script to `Makefile` or `justfile` for easy invocation

## 7. CI Integration
- [ ] 7.1 Create CI workflow file for benchmark execution
- [ ] 7.2 Configure to run on PR and main branch
- [ ] 7.3 Set up baseline storage (Criterion supports this)
- [ ] 7.4 Configure performance regression thresholds
- [ ] 7.5 Add CI job status badge to README
- [ ] 7.6 Set up benchmark result archiving (e.g., GitHub Pages, artifact storage)

## 8. Documentation
- [ ] 8.1 Document benchmark setup in README
- [ ] 8.2 Explain how to run benchmarks locally (`cargo bench`)
- [ ] 8.3 Document how to interpret Criterion output
- [ ] 8.4 Add comparison methodology documentation
- [ ] 8.5 Document allocation tracking approach
- [ ] 8.6 Create guide for adding new benchmarks
- [ ] 8.7 Document expected performance characteristics and trade-offs
