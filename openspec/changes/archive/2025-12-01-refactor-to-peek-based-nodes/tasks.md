# Tasks for refactor-to-peek-based-nodes

## Code Refactoring Tasks

- [x] Update `src/nodes/subscriber.rs` to use Element trait
  - [x] Change `AeronSubscriberNode` generic bound from `T: Clone + 'static` to `T: Element`
  - [x] Update `StreamPeekRef` impl to use `T: Element`
  - [x] Update MutableNode impl to use `T: Element`
  - [x] Update unit tests to verify Element types work correctly
  - [x] Add test with custom Element type (struct with Debug, Clone, Default)

- [x] Refactor `tests/summing_node_test.rs` to use peek pattern
  - [x] Import `AeronSubscriberNode` from `aerofoil::nodes`
  - [x] Update test documentation to explain peek-based composition
  - [x] Refactor `SummingNode` struct to accept upstream dependency
    - [x] Add `upstream: Rc<RefCell<T>>` field where `T: StreamPeekRef<i64>`
    - [x] Make SummingNode generic over upstream type `T`
    - [x] Remove direct `RusteronSubscriber` field
  - [x] Update `SummingNode::new()` to accept upstream parameter
  - [x] Refactor `SummingNode::process_upstream()` to use `peek_ref()`
    - [x] Replace direct polling with `self.upstream.borrow().peek_ref()`
    - [x] Implement change detection to track new values
    - [x] Keep running sum logic
  - [x] Update test setup to create both nodes
    - [x] Create `AeronSubscriberNode` with parser
    - [x] Create `Rc<RefCell<>>` wrapper manually
    - [x] Clone Rc for upstream reference
    - [x] Cast original Rc to `Rc<dyn Node>` for graph
    - [x] Create `SummingNode` with upstream reference
    - [x] Add both nodes to graph
  - [x] Add detailed comments explaining Rc<RefCell<>> pattern
  - [x] Verify test still passes with sum = 15

## Documentation Tasks

- [x] Add inline comments explaining the dual-Rc pattern
  - [x] Comment on why we create Rc<RefCell<>> manually
  - [x] Comment on cloning for upstream vs casting for graph
  - [x] Comment on Wingfoil's ownership requirements
- [x] Update module docs in `src/nodes/mod.rs` if needed
- [x] Update `AeronSubscriberNode` doc comments to mention Element trait
- [x] Add doc example showing peek-based composition pattern

## Validation Tasks

- [x] Run `cargo build --features rusteron`
- [x] Run `cargo test --lib` (unit tests pass)
- [x] Run `cargo test --features integration-tests` (integration test passes)
- [x] Run `cargo fmt`
- [x] Run `cargo clippy`
- [x] Validate with `openspec validate refactor-to-peek-based-nodes --strict`

## Notes

- Element trait = Debug + Clone + Default + 'static (from Wingfoil)
- The Rc<RefCell<>> pattern is necessary because:
  - We need a concrete type for the upstream reference
  - We need Rc<dyn Node> for the graph vector
  - Can't cast directly from concrete to dyn through into_node()
- This is the standard Wingfoil pattern for node composition
- SummingNode needs change detection since peek returns same reference
- Test should demonstrate clear separation: transport node → business logic node
