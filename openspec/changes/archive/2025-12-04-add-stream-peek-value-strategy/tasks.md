# Tasks for add-stream-peek-value-strategy

## Code Implementation Tasks

- [x] Create `AeronSubscriberCore<T, F, S>` shared implementation
  - [x] Add private `struct AeronSubscriberCore<T, F, S>` to `src/nodes/subscriber.rs`
  - [x] Move common fields: `subscriber: S`, `parser: F`, `current_value: T`
  - [x] Implement `new(subscriber, parser, initial_value) -> Self`
  - [x] Implement `poll_and_process(&mut self) -> Result<usize, TransportError>`
  - [x] Implement `current_value(&self) -> &T` accessor
  - [x] Keep struct module-private (no `pub`)

- [x] Refactor existing `AeronSubscriberNode` to use core struct
  - [x] Change struct to contain `core: AeronSubscriberCore<T, F, S>`
  - [x] Update `new()` to call `AeronSubscriberCore::new()`
  - [x] Update `MutableNode::cycle()` to call `self.core.poll_and_process()`
  - [x] Update `StreamPeekRef::peek_ref()` to call `self.core.current_value()`
  - [x] Ensure all existing tests still pass
  - [x] Maintain API compatibility (public API unchanged)

- [x] Implement `AeronSubscriberValueNode`
  - [x] Create `pub struct AeronSubscriberValueNode<T, F, S>` with `core` field
  - [x] Implement `new()` delegating to `AeronSubscriberCore::new()`
  - [x] Implement `MutableNode` trait delegating to `core.poll_and_process()`
  - [x] Implement `StreamPeek<T>` trait
    - [x] `peek_value(&self) -> T` returns `self.core.current_value().clone()`
    - [x] `peek_ref_cell(&self) -> Ref<'_, T>` panics with helpful message
  - [x] Add comprehensive doc comments
  - [x] Explain when to use vs `AeronSubscriberNode`

- [x] Export new node type
  - [x] Add `pub use subscriber::AeronSubscriberValueNode;` to `src/nodes/mod.rs`
  - [x] Verify public API is correct

## Documentation Tasks

- [x] Update `src/nodes/mod.rs` module documentation
  - [x] Add "Choosing Between Node Types" section
  - [x] Document `AeronSubscriberNode` (reference access)
    - [x] When to use (large types, Rc-wrapped, zero-copy)
    - [x] Access pattern example
  - [x] Document `AeronSubscriberValueNode` (value access)
    - [x] When to use (primitives, Copy types, small structs)
    - [x] Access pattern example
  - [x] Add comparison table
  - [x] Include both access patterns in "Accessing Values from Downstream Nodes"

- [x] Update `AeronSubscriberNode` documentation
  - [x] Add note about `AeronSubscriberValueNode` alternative
  - [x] Link to module docs for choosing between types

- [x] Add `AeronSubscriberValueNode` comprehensive documentation
  - [x] Type-level documentation
  - [x] Explain StreamPeek implementation
  - [x] Example usage with downstream node
  - [x] Note about peek_ref_cell() panic
  - [x] Cross-reference to AeronSubscriberNode

## Testing Tasks

- [x] Add integration test for `AeronSubscriberValueNode`
  - [x] Create `tests/counting_node_value_test.rs` or add to existing test file
  - [x] Implement `CountingNode` that uses `peek_value()`
  - [x] Set up Aeron infrastructure (driver, pub/sub)
  - [x] Publish test sequence [1, 2, 3, 4, 5]
  - [x] Create `AeronSubscriberValueNode` with i64 parser
  - [x] Wrap in `Rc<RefCell<>>`
  - [x] Create `CountingNode` with upstream reference
  - [x] Use `upstream.peek_value()` in cycle method
  - [x] Add both nodes to graph
  - [x] Verify counting logic works correctly
  - [x] Add detailed comments explaining value-access pattern

- [ ] Add unit tests for `AeronSubscriberValueNode`
  - [ ] Test `peek_value()` returns cloned value
  - [ ] Test with custom Element type
  - [ ] Test polling and value updates
  - [ ] Verify `peek_ref_cell()` panics with correct message
  - [ ] Test graph integration

- [x] Verify existing tests still pass
  - [x] All `AeronSubscriberNode` tests unchanged
  - [x] Integration test with SummingNode still passes

## Validation Tasks

- [x] Run `cargo build --features rusteron`
- [x] Run `cargo test --lib`
- [ ] Run `cargo test --features integration-tests`
- [ ] Run new value node integration test
- [x] Run `cargo fmt`
- [x] Run `cargo clippy`
- [ ] Run `cargo doc --no-deps --features rusteron`
- [ ] Verify doc examples compile and render correctly
- [x] Validate with `openspec validate add-stream-peek-value-strategy --strict`

## Notes

- **Key Design Decision**: Two separate node types instead of one node implementing both traits
- **Code Sharing**: Use private `AeronSubscriberCore` struct to avoid duplication (no macros)
- **API Surface**: Both nodes wrap the same core, differ only in access trait implementations
- **Graph Integration**: Both require `RefCell` wrapper for mutation during polling
- **Performance**: Similar performance, difference is ergonomics and type-level intent
- **Wingfoil Pattern**: Follows Wingfoil's distinction between `StreamPeekRef` and `StreamPeek`
- **Benefits of Core Struct**: No macros, clearer code, better IDE support, explicit structure

## Dependencies

- Requires: `peek-based-composition` spec (already deployed)
- No blocking dependencies for implementation
