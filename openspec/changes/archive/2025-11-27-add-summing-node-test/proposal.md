# Add Summing Node Test

## Why
Create a simple integration test demonstrating stateful stream processing with Wingfoil and Rusteron. A `SummingNode` will:
1. Wrap a Rusteron subscriber and register to be called back every Wingfoil cycle
2. Poll for messages containing binary i64 values
3. Maintain running sum state
4. Output the cumulative sum for each received input

This validates the core pattern for building stateful stream processors: non-blocking Aeron input polling combined with Wingfoil's stateful node model for processing logic like position keeping in HFT systems.

**Aligns with project conventions:**
- **Wingfoil for message processing**: Per project.md, use wingfoil for stateful stream processing
- **Zero-copy where possible**: Read i64 directly from received buffers without intermediate copies
- **Stateful processing**: Demonstrates position-keeping pattern (running sum is analogous to position state)
- **Non-blocking**: All polling operations return immediately
- **Integration test strategy**: Self-contained with automatic media driver management

## What Changes
- Add wingfoil dependency to Cargo.toml
- Create `SummingNode` that implements `MutableNode` trait with:
  - Wrapped `RusteronSubscriber`
  - Running sum state (i64)
  - `cycle()` method that polls for messages and updates sum
  - Output accessor to retrieve current sum and message count
- Create integration test that:
  - Starts media driver with `MediaDriverGuard`
  - Creates Rusteron publisher and subscriber
  - Publishes sequence of i64 values (e.g., 1, 2, 3, 4, 5)
  - Registers `SummingNode` in Wingfoil graph
  - Runs graph until all messages received
  - Verifies final sum is correct (e.g., 15 for inputs 1-5)
- Keep implementation simple (<150 lines total)

## Impact
- Affected specs: `summing-node-integration` (new capability)
- Affected code:
  - Adds wingfoil dependency to Cargo.toml
  - Creates `tests/summing_node_test.rs` integration test
  - Inline `SummingNode` struct in test file (not exposed as library code)
- Dependencies: Requires add-rusteron-adapter implementation (RusteronSubscriber, RusteronPublisher)
- Builds on: MediaDriverGuard from add-rusteron-adapter
- User value: Demonstrates stateful stream processing pattern for HFT position keeping
- Testing: Single integration test demonstrating the complete pattern
