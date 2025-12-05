# Tasks: Add Subscriber Node Builder

## Implementation Tasks

1. [ ] **Add `AeronSubscriberNodeBuilder` struct** in `src/nodes/builder.rs`
   - Generic over `T: Element`, `F: FnMut(&[u8]) -> Option<T>`, `S: AeronSubscriber`
   - Fields: `subscriber: Option<S>`, `parser: Option<F>`, `default_value: Option<T>`

2. [ ] **Implement fluent builder methods**
   - `.subscriber(s: S) -> Self`
   - `.parser(f: F) -> Self`
   - `.default(value: T) -> Self`

3. [ ] **Implement `build()` for `AeronSubscriberValueNode`**
   - Returns `(Rc<dyn Node>, Rc<RefCell<AeronSubscriberValueNode<T, F, S>>>)`
   - Panics with clear message if required fields missing

4. [ ] **Implement `build_ref()` for `AeronSubscriberValueRefNode`**
   - Returns `(Rc<dyn Node>, Rc<RefCell<AeronSubscriberValueRefNode<T, F, S>>>)`
   - Same validation as `build()`

5. [ ] **Add `builder()` associated functions to both node types**
   - `AeronSubscriberValueNode::builder() -> AeronSubscriberNodeBuilder`
   - `AeronSubscriberValueRefNode::builder() -> AeronSubscriberNodeBuilder`

6. [ ] **Add unit tests** in `src/nodes/builder.rs`
   - Test builder constructs valid node
   - Test builder returns correct tuple types
   - Test upstream reference can access values
   - Test graph node can be added to graph

7. [ ] **Update integration tests** to demonstrate builder pattern
   - Update `counting_node_value_test.rs` to use builder (optional comment showing comparison)

8. [ ] **Export builder from `src/nodes/mod.rs`**

## Validation

9. [ ] Run `cargo build` - verify compilation
10. [ ] Run `cargo test` - verify unit tests pass
11. [ ] Run `cargo test --features integration-tests` - verify integration tests
12. [ ] Run `cargo clippy` - verify no warnings
13. [ ] Run `openspec validate add-subscriber-node-builder --strict`
