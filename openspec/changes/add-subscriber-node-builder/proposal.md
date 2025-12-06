# Proposal: Add Subscriber Node Builder

## Summary

Add a builder pattern for `AeronSubscriberValueNode` and `AeronSubscriberValueRefNode` that eliminates the verbose `Rc<RefCell<>>` boilerplate.

## Motivation

Currently, integrating an Aeron subscriber node into a Wingfoil graph requires significant boilerplate:

```rust
// Current verbose pattern (from counting_node_value_test.rs)
let value_node = AeronSubscriberValueNode::new(subscriber, parser, 0i64);
let value_node_rc = Rc::new(RefCell::new(value_node));
let upstream_ref = value_node_rc.clone();
let value_graph_node: Rc<dyn Node> = value_node_rc;
```

This pattern is error-prone and obscures the actual intent. A builder simplifies this to:

```rust
// Ergonomic pattern - builder handles Rc<RefCell<>> wrapping
let node = AeronSubscriberValueNode::builder()
    .subscriber(subscriber)
    .parser(parser)
    .default(0i64)
    .build();

// Clone for graph (coerces to Rc<dyn Node>), use directly for upstream
let graph = Graph::new(vec![node.clone(), downstream], ...);
let downstream = MyNode::new(node, callback);
```

## Scope

- Add `AeronSubscriberNodeBuilder` struct with fluent API
- Implement `builder()` method on both node types
- Return `Rc<RefCell<Self>>` from `build()` / `build_ref()`
- Unit tests demonstrating the pattern

## Out of Scope

- Aeron connection/context management (users still create `RusteronSubscriber` externally)
- Publisher node builder (separate proposal)
- Graph-level builder (separate proposal)

## Related Specs

- `peek-based-composition` - Defines the dual-Rc pattern this builder encapsulates
- `peek-value-pattern` - Defines `AeronSubscriberValueNode` this builder constructs
- `transport-traits` - Defines `AeronSubscriber` trait used by the builder
