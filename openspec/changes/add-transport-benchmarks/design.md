# Transport Benchmarks Design

## Context
With both Rusteron and aeron-rs adapters implemented, we need objective performance data. Users choosing between backends need quantitative comparison. Additionally, benchmarks validate our zero-cost abstraction claims and help detect performance regressions.

**Prerequisites:**
- `add-transport-traits` (defines the API to benchmark)
- `add-rusteron-adapter` (provides first backend to benchmark)
- `add-aeron-rs-adapter` (provides second backend for comparison)

**Constraints:**
- Benchmarks must run in CI without excessive time overhead
- Need Aeron media driver running for meaningful results
- Must measure both backends fairly

**Stakeholders:**
- Users choosing between Rusteron and aeron-rs
- Maintainers detecting performance regressions
- Contributors optimizing implementations

## Goals / Non-Goals

**Goals:**
- Quantitative latency and throughput comparison of backends
- Verify zero-copy behavior empirically
- Statistical rigor via Criterion
- CI integration for regression detection
- Clear documentation for interpreting results

**Non-Goals:**
- Benchmarking against other messaging systems (just Aeron backends)
- End-to-end system benchmarks (focused on adapter layer)
- Production load testing (that's for users to do)

## Decisions

### Decision 1: Criterion.rs for benchmarking
**What:** Use Criterion as the benchmarking framework.

**Why:**
- Industry standard for Rust benchmarking
- Statistical analysis built-in (outlier detection, confidence intervals)
- Baseline comparison and regression detection
- HTML report generation
- Well-documented and maintained

**Alternatives considered:**
- **cargo bench (libtest):** Less sophisticated statistics, no regression detection - rejected
- **Custom harness:** Reinventing the wheel - rejected

### Decision 2: Separate benchmark runs per backend
**What:** Run benchmarks twice - once with `--features rusteron`, once with `--features aeron-rs`.

**Why:**
- Features are mutually exclusive (can't build both simultaneously)
- Allows fair comparison (same code, different backend)
- Simpler than trying to conditionally compile both

**Approach:**
```bash
cargo bench --features rusteron
cargo bench --features aeron-rs
# Then compare results
```

### Decision 3: Key metrics to measure
**What:** Focus on three primary metrics:
1. **Publication latency** (time from `offer()` call to completion)
2. **Subscription throughput** (messages/second received)
3. **Allocation count** (heap allocations in hot path)

**Why:**
- Latency is critical for HFT use case
- Throughput shows scalability
- Allocations validate zero-copy claims
- These are the most meaningful differentiators

**Not measuring (at least initially):**
- CPU utilization (depends heavily on system state)
- Memory footprint (mostly determined by Aeron buffer config)
- Network bandwidth (Aeron's concern, not adapter's)

### Decision 4: Allocation tracking via dhat
**What:** Use `dhat` (or similar) to track heap allocations during benchmark execution.

**Why:**
- Proves zero-copy behavior empirically
- Catches accidental allocations introduced by changes
- Easy to integrate with Criterion benchmarks

**Acceptance criteria:** Zero heap allocations in `offer()` and `poll()` hot paths when using zero-copy APIs.

### Decision 5: Benchmark against mock Aeron media driver or real one
**What:** Benchmarks run against real Aeron media driver (not mocks).

**Why:**
- Mocks don't reflect real performance characteristics
- Need to measure actual Aeron client overhead
- Realistic back-pressure and buffer management behavior

**Trade-off:** Requires media driver running in CI. Will document setup clearly.

**Alternative for quick local testing:** Could provide mock-based micro-benchmarks, but they're not the primary comparison.

### Decision 6: CI regression detection with thresholds
**What:** Configure Criterion to fail CI if performance regresses beyond threshold (e.g., 10% slower).

**Why:**
- Catches accidental performance degradation
- Encourages performance-conscious development
- Prevents slow drift

**Implementation:** Criterion's `--save-baseline` and `--baseline` flags support this.

## Risks / Trade-offs

**Risk:** Benchmarks in CI may be noisy due to shared hardware
- **Impact:** False positives on regression detection
- **Mitigation:**
  - Use Criterion's outlier detection
  - Set reasonable regression thresholds (10%, not 1%)
  - Run benchmarks on dedicated CI runners if available
  - Allow manual re-run on suspected false positives

**Risk:** Benchmarks require Aeron media driver running
- **Impact:** CI setup complexity
- **Mitigation:**
  - Document media driver setup clearly
  - Provide Docker container or script for CI
  - Consider making benchmark CI job optional (manual trigger)

**Trade-off:** Separate runs per backend vs combined report
- **Benefit:** Can't build both simultaneously (mutual exclusivity)
- **Cost:** Manual comparison step needed
- **Solution:** Script to run both and generate comparison table

**Risk:** Benchmarks may take significant time
- **Impact:** Slow CI feedback loop
- **Mitigation:**
  - Run benchmarks in separate CI job (don't block main tests)
  - Configure Criterion for shorter runs in CI, longer locally
  - Only run on main branch and PRs labeled "performance"

## Migration Plan

**Prerequisites:**
1. Both adapters must be implemented and working
2. CI infrastructure must support running Aeron media driver

**Deployment:**
1. Add benchmarks in this change
2. Run initial baseline benchmarks for both backends
3. Document results
4. Enable CI regression detection

**Baseline establishment:**
- First run establishes baseline
- Future runs compare against it
- Re-baseline when intentional performance changes made

**Rollback:**
Benchmarks are non-invasive (dev-dependency only). Can disable CI job if problematic.

## Open Questions

1. **What regression threshold should we use?**
   - 5% too sensitive for noisy CI?
   - 10% reasonable starting point?
   - 20% too permissive?
   - **Lean toward: 10% initially, tune based on experience**

2. **Should we publish benchmark results publicly?**
   - Could host on GitHub Pages
   - Helps users make informed decisions
   - Might create pressure if one backend significantly slower
   - **Lean toward: Yes, transparency is good. Users need data.**

3. **Do we need separate benchmarks for different message sizes?**
   - Small messages (e.g., 64 bytes) for latency-sensitive use cases
   - Large messages (e.g., 8KB) for throughput-oriented use cases
   - **Decision: Yes, benchmark both. Create categories.**

4. **Should benchmarks test back-pressure scenarios?**
   - Important for real-world behavior
   - How do backends handle buffer full conditions?
   - **Lean toward: Yes, add in Phase 2 if time permits**

## Benchmark Categories

### Category 1: Minimal Latency (Small Messages)
- Message size: 64 bytes
- Metric: p50, p99, p999 latency
- Use case: Tick data, order updates

### Category 2: High Throughput (Medium Messages)
- Message size: 512 bytes
- Metric: Messages/second
- Use case: Market data snapshots

### Category 3: Large Payload (Big Messages)
- Message size: 8KB
- Metric: Throughput (MB/s)
- Use case: Historical data dumps

### Category 4: Zero-Copy Verification
- All message sizes
- Metric: Allocation count (must be zero)
- Use case: Validating zero-copy claims
