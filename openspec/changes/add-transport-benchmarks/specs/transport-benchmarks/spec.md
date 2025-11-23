# Transport Benchmarks

## ADDED Requirements

### Requirement: Performance Benchmarking
The library SHALL provide benchmarks comparing the performance characteristics of transport adapter implementations.

#### Scenario: Publication latency benchmark
- **WHEN** benchmarks are run for available backends
- **THEN** publication latency (time from offer to confirmation) is measured and compared

#### Scenario: Subscription throughput benchmark
- **WHEN** benchmarks are run for available backends
- **THEN** message reception throughput (messages per second) is measured and compared

#### Scenario: Zero-copy verification
- **WHEN** benchmarks measure memory allocations
- **THEN** zero-copy paths can be verified to have no heap allocations in the critical path

### Requirement: Benchmark Framework
The library SHALL use Criterion.rs for statistical benchmarking with regression detection.

#### Scenario: Statistical rigor
- **WHEN** benchmarks are executed
- **THEN** results include confidence intervals, outlier detection, and statistical significance

#### Scenario: Baseline comparison
- **WHEN** benchmarks are run after code changes
- **THEN** performance is compared against previous baseline to detect regressions

### Requirement: Backend Comparison
The library SHALL benchmark both Rusteron and aeron-rs adapters for direct comparison.

#### Scenario: Rusteron benchmarks
- **WHEN** benchmarks are built with `--features rusteron`
- **THEN** Rusteron adapter performance is measured

#### Scenario: Aeron-rs benchmarks
- **WHEN** benchmarks are built with `--features aeron-rs`
- **THEN** aeron-rs adapter performance is measured

#### Scenario: Comparison report
- **WHEN** benchmarks for both backends are available
- **THEN** a comparison report can be generated showing relative performance

### Requirement: Allocation Tracking
The library SHALL track heap allocations during benchmark execution to verify zero-copy behavior.

#### Scenario: Zero-allocation publication
- **WHEN** publishing messages using zero-copy path (try_claim)
- **THEN** benchmarks verify no heap allocations occur in the publication hot path

#### Scenario: Zero-allocation subscription
- **WHEN** receiving messages via subscription
- **THEN** benchmarks verify no heap allocations occur in the message handling hot path

### Requirement: CI Integration
The library SHALL run benchmarks in continuous integration to detect performance regressions.

#### Scenario: Regression detection
- **WHEN** code changes are pushed
- **THEN** CI runs benchmarks and fails if significant performance regression is detected

#### Scenario: Benchmark result archiving
- **WHEN** benchmarks complete in CI
- **THEN** results are stored for historical comparison and trend analysis
