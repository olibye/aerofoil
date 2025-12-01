# Proposal: Refactor to Peek-Based Nodes with Element Types

## Problem

The current implementation has `AeronSubscriberNode` that implements `StreamPeekRef<T>`, but the integration test (`summing_node_test.rs`) doesn't use it. Instead, the test uses the old pattern where `SummingNode` directly owns a `RusteronSubscriber` and polls it.

This creates several issues:

1. **Not demonstrating the intended pattern**: The `AeronSubscriberNode` was designed to separate transport from business logic, but tests don't show this separation
2. **Mixed concerns**: `SummingNode` mixes Aeron polling with sum calculation logic
3. **Not using Wingfoil Element pattern**: Current implementation uses arbitrary types with Clone, but doesn't follow Wingfoil's `Element` trait convention
4. **Difficult manual Rc<RefCell<>> management**: The peek pattern implementation encountered issues with Rust's ownership and the Wingfoil graph API

## Solution

Refactor both `AeronSubscriberNode` and the integration test to properly demonstrate the peek-based pattern with Wingfoil Element types:

1. **Update AeronSubscriberNode to use Element**: Change generic bound from `T: Clone + 'static` to `T: Element` (which implies `Debug + Clone + Default + 'static`)

2. **Simplify graph integration**: Use Wingfoil's `into_node()` correctly by not pre-wrapping in `Rc<RefCell<>>`

3. **Update SummingNode to use peek**: Refactor to accept upstream `Stream<T>` and use `peek_ref()` to access values

4. **Demonstrate proper pattern in tests**: Show how to compose transport nodes with business logic nodes

## Implementation Approach

### AeronSubscriberNode Changes

```rust
// Before: arbitrary Clone types
impl<T, F, S> AeronSubscriberNode<T, F, S>
where
    T: Clone + 'static,
    F: FnMut(&[u8]) -> Option<T> + 'static,
    S: AeronSubscriber + 'static,
{ }

// After: use Element
impl<T, F, S> AeronSubscriberNode<T, F, S>
where
    T: Element,  // = Debug + Clone + Default + 'static
    F: FnMut(&[u8]) -> Option<T> + 'static,
    S: AeronSubscriber + 'static,
{ }
```

### SummingNode Pattern

The key insight from the earlier implementation struggle: **Don't manually manage `Rc<RefCell<>>`**. Let Wingfoil's `into_node()` handle it:

```rust
// SummingNode with generic upstream
struct SummingNode<T, F>
where
    T: StreamPeekRef<i64>,
    F: FnMut(SummingNodeOutput),
{
    upstream: Rc<RefCell<T>>,  // Reference to upstream node
    running_sum: i64,
    // ...
}

// Graph construction - the tricky part solved:
let subscriber_node = AeronSubscriberNode::new(subscriber, parser, 0);
let subscriber_rc = Rc::new(RefCell::new(subscriber_node));

// Clone for upstream reference (keeps concrete type)
let upstream_ref = subscriber_rc.clone();

// Cast to Rc<dyn Node> for graph (erases type)
let graph_node: Rc<dyn Node> = subscriber_rc;

let summing_node = SummingNode::new(upstream_ref, callback);

// Both nodes in graph
let graph = Graph::new(
    vec![
        graph_node,           // Subscriber as dyn Node
        summing_node.into_node(), // Summing as dyn Node
    ],
    RunMode::RealTime,
    RunFor::Cycles(10)
);
```

## Why

**Demonstrates proper Wingfoil patterns**: Shows how to compose nodes using `peek()` and `Stream` traits

**Element types are Wingfoil convention**: Using `Element` trait bound makes code consistent with Wingfoil ecosystem

**Separation of concerns**: Transport nodes (AeronSubscriberNode) separate from business logic nodes (SummingNode)

**Reusability**: `AeronSubscriberNode` becomes a reusable component for any Aeron input

**Testability**: Business logic nodes can be tested with mock streams without Aeron infrastructure

## Scope

### In Scope

- Update `AeronSubscriberNode` generic bounds to use `Element`
- Refactor `summing_node_test.rs` to use `AeronSubscriberNode` with peek pattern
- Update `SummingNode` to accept upstream `Rc<RefCell<T>>` where `T: StreamPeekRef<i64>`
- Document the Rc<RefCell<>> pattern for sharing nodes between graph and upstream references
- Ensure test still passes with refactored architecture
- Add inline comments explaining the ownership pattern

### Out of Scope

- Changing AeronSubscriberNode's core functionality (just updating type bounds)
- Adding new features to AeronSubscriberNode
- Supporting multiple upstream nodes (single upstream only)
- Alternative patterns (this is the standard Wingfoil way)

## Dependencies

- Existing `AeronSubscriberNode` implementation in `src/nodes/subscriber.rs`
- Existing `summing_node_test.rs` integration test
- Wingfoil's `Element` trait and `StreamPeekRef` trait

## Risks

**Complexity of Rc<RefCell<>> pattern**: This was the main stumbling block in the original implementation. The proposal now documents the correct pattern clearly.

**Migration**: Users who may have started using the old pattern will need to update. However, since this is early in the project lifecycle, impact is minimal.

## Alternatives Considered

**Alternative 1: Keep current direct polling pattern**
- Rejected: Doesn't demonstrate proper Wingfoil node composition

**Alternative 2: Use trait objects everywhere (Rc<RefCell<dyn StreamPeekRef<T>>>)**
- Rejected: Causes issues with Wingfoil's `into_node()` which needs concrete `MutableNode` types. The dual-Rc pattern (one concrete for upstream, one dyn Node for graph) is the correct approach.

**Alternative 3: Don't use Element, keep arbitrary Clone**
- Rejected: Inconsistent with Wingfoil conventions. Element adds Debug and Default which are useful.

## Success Criteria

1. `AeronSubscriberNode` uses `Element` trait bound
2. Integration test demonstrates full peek-based pattern
3. `SummingNode` successfully uses `upstream.peek_ref()` to access values
4. Test passes and produces correct sum (15 for inputs 1,2,3,4,5)
5. Code includes clear comments explaining the Rc<RefCell<>> pattern
6. Documentation shows proper way to compose transport and business logic nodes
