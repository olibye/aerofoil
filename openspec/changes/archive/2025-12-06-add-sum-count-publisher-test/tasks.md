# Tasks: Add Sum/Count Publisher Integration Test

## Implementation Tasks

1. [x] **Create `tests/sum_count_publisher_test.rs`**
   - Start media driver
   - Create input publisher and subscriber (stream 2001)
   - Create sum output publisher (stream 2002) and count output publisher (stream 2003)
   - Create output subscribers to verify published values
   - Build `AeronSubscriberValueNode` with builder
   - Create `SummingNode` with callback that captures sum publisher
   - Create `CountingNode` with callback that captures count publisher
   - Both nodes share the same upstream subscriber (fan-out pattern)
   - Publish test values [1, 2, 3, 4, 5]
   - Run graph
   - Verify sum stream received 15 (1+2+3+4+5)
   - Verify count stream received 5

## Validation

2. [x] Run `cargo build` - verify compilation
3. [x] Run `cargo test --test sum_count_publisher_test`
4. [x] Run `cargo clippy` - no warnings
5. [x] Run `openspec validate add-sum-count-publisher-test --strict`
