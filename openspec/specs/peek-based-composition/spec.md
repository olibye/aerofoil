# peek-based-composition Specification

## Purpose

This specification defines the peek-based node composition pattern for Aerofoil, which enables clean separation between transport layer nodes (e.g., AeronSubscriberValueRefNode) and business logic nodes (e.g., SummingNode) using Wingfoil's StreamPeekRef trait. The pattern allows downstream nodes to access upstream values via peek_ref() while maintaining proper ownership semantics through the dual-Rc pattern for graph integration.
## Requirements
### Requirement: AeronSubscriberValueRefNode Element Type Constraint

AeronSubscriberValueRefNode SHALL use Wingfoil's Element trait bound for message types to ensure compatibility with Wingfoil's type system and enable Debug, Clone, Default, and 'static guarantees.

#### Scenario: Given AeronSubscriberValueRefNode when created with Element type then compiles successfully

```rust
use wingfoil::types::Element;

#[derive(Debug, Clone, Default)]
struct Trade {
    price: f64,
    quantity: i64,
}

// Element is automatically implemented for types that are Debug + Clone + Default + 'static
// This should compile without explicit Element impl
let parser = |fragment: &[u8]| -> Option<Trade> {
    // Parse Trade from bytes
    Some(Trade::default())
};

let node = AeronSubscriberValueRefNode::new(subscriber, parser, Trade::default());
// Should compile because Trade: Element
```

#### Scenario: Given AeronSubscriberValueRefNode when created with i64 then works with primitive Element types

```rust
// i64 implements Element (Debug + Clone + Default + 'static)
let parser = |fragment: &[u8]| -> Option<i64> {
    if fragment.len() >= 8 {
        Some(i64::from_le_bytes(fragment[0..8].try_into().ok()?))
    } else {
        None
    }
};

let node = AeronSubscriberValueRefNode::new(subscriber, parser, 0i64);
// Should compile because i64: Element
```

### Requirement: Peek-Based Downstream Node Pattern

Downstream nodes SHALL accept upstream dependencies as Rc<RefCell<T>> where T implements StreamPeekRef to enable access to upstream values via peek_ref().

#### Scenario: Given SummingNode when created with upstream StreamPeekRef then can peek values

```rust
// SummingNode is generic over upstream type
struct SummingNode<T, F>
where
    T: StreamPeekRef<i64>,
    F: FnMut(SummingNodeOutput),
{
    upstream: Rc<RefCell<T>>,
    running_sum: i64,
    message_count: usize,
    last_value: i64,
    output_callback: F,
}

// Given: Create upstream subscriber node
let subscriber_node = AeronSubscriberValueRefNode::new(subscriber, parser, 0);
let subscriber_rc = Rc::new(RefCell::new(subscriber_node));

// When: Create downstream node with upstream reference
let summing_node = SummingNode::new(subscriber_rc.clone(), callback);

// Then: Can use peek_ref() in cycle method
fn cycle(&mut self, state: &mut GraphState) -> bool {
    let current_value = *self.upstream.borrow().peek_ref();
    // Process current_value
    // ...
}
```

#### Scenario: Given downstream node when upstream value changes then detects change via peek

```rust
// Given: Downstream node tracking last seen value
let mut node = SummingNode::new(upstream_rc, callback);

// When: Upstream receives new values (1, then 2)
// Cycle 1: upstream has value 1
let should_stop = node.cycle(&mut state);
assert_eq!(node.last_value, 1);
assert_eq!(node.running_sum, 1);

// Cycle 2: upstream has value 2
let should_stop = node.cycle(&mut state);
assert_eq!(node.last_value, 2);
assert_eq!(node.running_sum, 3); // 1 + 2

// Then: Change detection works by comparing peek_ref() to last_value
```

### Requirement: Dual Rc Pattern for Graph Integration

Wingfoil graphs SHALL support nodes that are both in the graph and referenced by other nodes by using the dual-Rc pattern where one Rc<RefCell<T>> is cloned for upstream references and another is cast to Rc<dyn Node> for the graph.

#### Scenario: Given node that needs to be both in graph and upstream reference when using dual-Rc pattern then integrates successfully

```rust
// Given: Create node that will be in graph AND referenced by downstream node
let subscriber_node = AeronSubscriberValueRefNode::new(subscriber, parser, 0);

// Create Rc<RefCell<>> manually (don't use into_node() yet)
let subscriber_rc = Rc::new(RefCell::new(subscriber_node));

// When: Clone for upstream reference (keeps concrete type)
let upstream_ref = subscriber_rc.clone();

// And: Cast to Rc<dyn Node> for graph (erases type)
let graph_node: Rc<dyn Node> = subscriber_rc;

// Then: Can use both - upstream_ref for SummingNode, graph_node for Graph
let summing_node = SummingNode::new(upstream_ref, callback);

let mut graph = Graph::new(
    vec![
        graph_node,              // Subscriber as dyn Node
        summing_node.into_node(), // Summing as dyn Node
    ],
    RunMode::RealTime,
    RunFor::Cycles(10)
);

// Graph executes successfully
graph.run().expect("Graph execution should succeed");
```

### Requirement: Integration Test Demonstrates Peek Pattern

The summing_node_test integration test SHALL demonstrate the peek-based composition pattern with AeronSubscriberValueRefNode and SummingNode to serve as reference implementation for users.

#### Scenario: Given integration test when run then demonstrates complete peek-based flow

```rust
#[test]
fn given_aeron_messages_when_summing_node_processes_then_calculates_correct_sum() {
    // Given: Aeron infrastructure setup
    let _driver = MediaDriverGuard::start().expect("Media driver");
    let subscription = /* create Rusteron subscription */;
    let publication = /* create Rusteron publication */;

    // Publish test messages [1, 2, 3, 4, 5]
    for value in &[1i64, 2, 3, 4, 5] {
        publication.offer(&value.to_le_bytes()).expect("Offer");
    }

    // When: Create AeronSubscriberValueRefNode (transport layer)
    let parser = |fragment: &[u8]| -> Option<i64> {
        if fragment.len() >= 8 {
            Some(i64::from_le_bytes(fragment[0..8].try_into().ok()?))
        } else {
            None
        }
    };
    let subscriber_node = AeronSubscriberValueRefNode::new(
        RusteronSubscriber::new(subscription),
        parser,
        0
    );

    // Create dual-Rc pattern
    let subscriber_rc = Rc::new(RefCell::new(subscriber_node));
    let upstream: Rc<dyn Node> = subscriber_rc.clone();

    // Create SummingNode (business logic layer)
    let outputs = Rc::new(RefCell::new(Vec::new()));
    let outputs_clone = outputs.clone();
    let callback = move |output| outputs_clone.borrow_mut().push(output);
    let summing_node = SummingNode::new(subscriber_rc.clone(), callback);

    // Add both to graph
    let mut graph = Graph::new(
        vec![upstream, summing_node.into_node()],
        RunMode::RealTime,
        RunFor::Cycles(10)
    );

    // Then: Graph executes and calculates correct sum
    graph.run().expect("Graph execution");
    let final_output = outputs.borrow().last().unwrap();
    assert_eq!(final_output.sum, 15); // 1+2+3+4+5
    assert_eq!(final_output.count, 5);
}
```

### Requirement: Change Detection in Peek Pattern

Downstream nodes using peek SHALL implement change detection by comparing peeked values to previously seen values to identify when upstream has new data.

#### Scenario: Given SummingNode when value hasn't changed then doesn't re-sum

```rust
// Given: SummingNode with last_value = 5, running_sum = 15
let mut node = SummingNode {
    upstream: upstream_rc,
    running_sum: 15,
    message_count: 3,
    last_value: 5,
    output_callback: callback,
};

// When: Upstream still has value 5 (no new message)
// peek_ref() returns 5, same as last_value
node.process_upstream();

// Then: Sum unchanged, count unchanged
assert_eq!(node.running_sum, 15);
assert_eq!(node.message_count, 3);
```

#### Scenario: Given SummingNode when value changed then adds to sum

```rust
// Given: SummingNode with last_value = 5, running_sum = 15
let mut node = SummingNode {
    upstream: upstream_rc,
    running_sum: 15,
    message_count: 3,
    last_value: 5,
    output_callback: callback,
};

// When: Upstream has new value 10 (new message arrived)
// peek_ref() returns 10, different from last_value
node.process_upstream();

// Then: Sum updated, count incremented
assert_eq!(node.running_sum, 25); // 15 + 10
assert_eq!(node.message_count, 4);
assert_eq!(node.last_value, 10);
```

