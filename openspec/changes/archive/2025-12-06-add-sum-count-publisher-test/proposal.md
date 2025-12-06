# Proposal: Add Sum/Count Publisher Integration Test

## Why

Demonstrates the fan-out pattern: a single `AeronSubscriberValueNode` feeding multiple downstream processing nodes (`SummingNode` and `CountingNode`), each publishing to separate Aeron output streams. This validates the publisher callback pattern and shows idiomatic wingfoil graph composition with shared upstream.

## What Changes

Add an integration test that:
1. Subscribes to an Aeron stream of i64 values using `AeronSubscriberValueNode`
2. Shares the subscriber node with both `SummingNode` and `CountingNode` (fan-out)
3. Each node's output callback captures an `AeronPublisher` and publishes results
4. Verifies the published values by subscribing to the output streams

## Scope

- New integration test `tests/sum_count_publisher_test.rs`
- Reuses existing `SummingNode` and `CountingNode` from `tests/common/mod.rs`
- Demonstrates publisher-in-callback pattern

## Out of Scope

- New node types (reuse existing)
- Publisher node builder (separate proposal)
