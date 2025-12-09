# Implementation Tasks

## 1. Publication Latency Benchmark Cleanup
- [x] 1.1 Remove `bench_offer_bare` function
- [x] 1.2 Remove `bench_try_claim_aerofoil` function (not implemented)
- [x] 1.3 Rename `bench_offer_aerofoil` to `bench_offer`
- [x] 1.4 Rename benchmark group from "offer/aerofoil" to "offer"
- [x] 1.5 Update module documentation

## 2. Subscription Throughput Benchmark Cleanup
- [x] 2.1 Remove `bench_poll_bare` function
- [x] 2.2 Rename `bench_poll_aerofoil` to `bench_poll`
- [x] 2.3 Rename benchmark group from "poll/aerofoil" to "poll"
- [x] 2.4 Update module documentation

## 3. Allocation Tracking Benchmark Cleanup
- [x] 3.1 Remove `bench_try_claim_allocations` function
- [x] 3.2 Update criterion_group to remove try_claim

## 4. Validation
- [x] 4.1 Run `cargo build --benches --features embedded-driver`
- [x] 4.2 Run `cargo bench --features embedded-driver -- --list` to verify benchmark structure
