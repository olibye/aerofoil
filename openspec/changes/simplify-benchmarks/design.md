# Benchmark Simplification Design

## Current State

The benchmarks have two layers of comparison that are now obsolete:

1. **"bare" vs "aerofoil"** - Compares direct rusteron API calls vs calls through aerofoil's trait abstraction. This was useful when comparing the overhead of the abstraction layer, but the overhead is negligible (<1ns) and the comparison adds no value.

2. **Multi-backend structure** - The code was structured to support rusteron and aeron-rs benchmarks side by side. Now there's only rusteron.

### Current Benchmark Groups

| File | Group | Purpose | Keep? |
|------|-------|---------|-------|
| publication_latency.rs | offer/bare | Direct rusteron offer | Remove |
| publication_latency.rs | offer/aerofoil | Aerofoil offer | Keep (rename to "offer") |
| publication_latency.rs | try_claim/aerofoil | Zero-copy claim | Remove (not implemented) |
| subscription_throughput.rs | poll/bare | Direct rusteron poll | Remove |
| subscription_throughput.rs | poll/aerofoil | Aerofoil poll | Keep (rename to "poll") |
| subscription_throughput.rs | poll_parse | Poll with parsing | Keep |
| subscription_throughput.rs | poll_burst | Burst throughput | Keep |
| transceiver.rs | transceiver/* | Pub/sub scenarios | Keep |
| allocation_tracking.rs | allocations/* | Allocation verification | Keep (remove try_claim) |

## Decisions

### Remove "bare" benchmarks
The trait abstraction has negligible overhead. Benchmarking both "bare" and "aerofoil" produces nearly identical numbers and doubles the benchmark time without adding insight.

### Remove try_claim benchmarks
`try_claim` is not implemented (returns `Err`). Benchmarking an unimplemented feature is misleading.

### Simplify naming
Remove "/aerofoil" suffix since there's only one implementation path. "offer" is clearer than "offer/aerofoil".

### Keep practical benchmarks
- `offer` - Core publish latency
- `poll` - Core subscribe throughput
- `poll_parse` - Realistic workload with message parsing
- `poll_burst` - Burst performance characteristics
- `transceiver/*` - Combined pub/sub scenarios
- `allocations/publication` and `allocations/subscription` - Zero-allocation verification
