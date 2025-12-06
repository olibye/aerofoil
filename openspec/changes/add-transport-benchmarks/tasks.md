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

## 4. Combined Pub/Sub (Transceiver) Benchmarks
- [ ] 4.1 Create `benches/transceiver.rs`
- [ ] 4.2 Write benchmark for simultaneous publish/subscribe on different streams
- [ ] 4.3 Write benchmark for request/response roundtrip latency
- [ ] 4.4 Write benchmark for bidirectional symmetric exchange
- [ ] 4.5 Test with different message sizes and rates
- [ ] 4.6 Measure publish latency degradation under concurrent subscribe load
- [ ] 4.7 Measure subscribe latency degradation under concurrent publish load
- [ ] 4.8 Compare Rusteron vs aeron-rs transceiver performance

## 5. Allocation Tracking
- [ ] 5.1 Create `benches/allocation_tracking.rs`
- [ ] 5.2 Set up allocation profiler integration
- [ ] 5.3 Verify zero allocations in Rusteron publication hot path
- [ ] 5.4 Verify zero allocations in aeron-rs publication hot path
- [ ] 5.5 Verify zero allocations in Rusteron subscription hot path
- [ ] 5.6 Verify zero allocations in aeron-rs subscription hot path
- [ ] 5.7 Document any unavoidable allocations and their causes

## 6. Benchmark Configuration
- [ ] 6.1 Configure Criterion settings (warm-up, measurement time, sample size)
- [ ] 6.2 Set up feature-gated benchmarks for each backend
- [ ] 6.3 Create benchmark helper functions to reduce duplication
- [ ] 6.4 Add command-line options for selective benchmark execution

## 7. Comparison Tooling
- [ ] 7.1 Create script to run benchmarks for both backends sequentially
- [ ] 7.2 Parse Criterion output and generate comparison report
- [ ] 7.3 Create markdown table or chart comparing results
- [ ] 7.4 Add script to `Makefile` or `justfile` for easy invocation

## 8. CI Integration
- [ ] 8.1 Create CI workflow file for benchmark execution
- [ ] 8.2 Configure to run on PR and main branch
- [ ] 8.3 Set up baseline storage (Criterion supports this)
- [ ] 8.4 Configure performance regression thresholds
- [ ] 8.5 Add CI job status badge to README
- [ ] 8.6 Set up benchmark result archiving (e.g., GitHub Pages, artifact storage)

## 9. Documentation
- [ ] 9.1 Document benchmark setup in README
- [ ] 9.2 Explain how to run benchmarks locally (`cargo bench`)
- [ ] 9.3 Document how to interpret Criterion output
- [ ] 9.4 Add comparison methodology documentation
- [ ] 9.5 Document allocation tracking approach
- [ ] 9.6 Create guide for adding new benchmarks
- [ ] 9.7 Document expected performance characteristics and trade-offs
