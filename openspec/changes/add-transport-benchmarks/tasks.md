# Implementation Tasks

## 1. Benchmark Framework Setup
- [x] 1.1 Add `criterion` to `[dev-dependencies]` in Cargo.toml
- [x] 1.2 Add `dhat` or similar allocation profiler to dev-dependencies
- [x] 1.3 Create `benches/` directory
- [x] 1.4 Configure `[[bench]]` entries in Cargo.toml

## 2. Publication Latency Benchmarks
- [x] 2.1 Create `benches/publication_latency.rs`
- [x] 2.2 Write benchmark for Rusteron publication latency (offer method)
- [x] 2.3 Write benchmark for aeron-rs publication latency (offer method)
- [x] 2.4 Write benchmark for Rusteron zero-copy publication (try_claim)
- [x] 2.5 Write benchmark for aeron-rs zero-copy publication (try_claim)
- [x] 2.6 Add different message size scenarios (small, medium, large)

## 3. Subscription Throughput Benchmarks
- [x] 3.1 Create `benches/subscription_throughput.rs`
- [x] 3.2 Write benchmark for Rusteron subscription throughput
- [x] 3.3 Write benchmark for aeron-rs subscription throughput
- [x] 3.4 Test with different message rates (burst mode benchmark)
- [x] 3.5 Add poll with parsing benchmark

## 4. Combined Pub/Sub (Transceiver) Benchmarks
- [x] 4.1 Create `benches/transceiver.rs`
- [x] 4.2 Write benchmark for simultaneous publish/subscribe on different streams
- [x] 4.3 Write benchmark for request/response roundtrip latency
- [x] 4.4 Write benchmark for bidirectional symmetric exchange
- [x] 4.5 Test with different message sizes
- [x] 4.8 Compare Rusteron vs aeron-rs transceiver performance (feature-gated)

## 5. Allocation Tracking
- [x] 5.1 Create `benches/allocation_tracking.rs`
- [x] 5.2 Set up allocation profiler integration (dhat feature flag)
- [x] 5.3 Benchmark Rusteron publication hot path allocations
- [x] 5.4 Benchmark aeron-rs publication hot path allocations
- [x] 5.5 Benchmark Rusteron subscription hot path allocations
- [x] 5.6 Benchmark aeron-rs subscription hot path allocations
- [x] 5.7 Benchmark try_claim hot path allocations

## 6. Benchmark Configuration
- [x] 6.1 Configure Criterion settings (html_reports feature)
- [x] 6.2 Set up feature-gated benchmarks for each backend
- [x] 6.3 Create benchmark helper functions (common/mod.rs)
- [x] 6.4 Add MessageSize enum for consistent size testing

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
