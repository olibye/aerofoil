# peek-value-pattern Specification

## Purpose
TBD - created by archiving change add-stream-peek-value-strategy. Update Purpose after archive.
## Requirements
### Requirement: AeronSubscriberValueNode Implementation

Aerofoil SHALL provide `AeronSubscriberValueNode<T, F, S>` that implements `StreamPeek<T>` for value-based access to cheap-to-clone types, alongside the existing `AeronSubscriberValueRefNode<T, F, S>` that implements `StreamPeekRef<T>` for reference-based access.

> **Related**: Complements `peek-based-composition` spec by adding value-access strategy

#### Scenario: Given AeronSubscriberValueNode when created then implements StreamPeek trait

```rust
use aerofoil::nodes::AeronSubscriberValueNode;
use aerofoil::transport::rusteron::RusteronSubscriber;
use wingfoil::StreamPeek;

// Given: Parser for i64 messages
let parser = |fragment: &[u8]| -> Option<i64> {
    if fragment.len() >= 8 {
        Some(i64::from_le_bytes(fragment[0..8].try_into().ok()?))
    } else {
        None
    }
};

// When: Create AeronSubscriberValueNode
let subscriber = RusteronSubscriber::new(subscription);
let node = AeronSubscriberValueNode::new(subscriber, parser, 0i64);

// Then: Node implements StreamPeek<i64>
let node_rc = Rc::new(RefCell::new(node));
let value: i64 = node_rc.peek_value(); // Compiles - StreamPeek is implemented
assert_eq!(value, 0); // Initial value
```

#### Scenario: Given AeronSubscriberValueNode when wrapped in RefCell then peek_value returns by value

```rust
// Given: AeronSubscriberValueNode wrapped for graph integration
let node = AeronSubscriberValueNode::new(subscriber, parser, 42i64);
let node_rc = Rc::new(RefCell::new(node));

// When: Calling peek_value()
let value: i64 = node_rc.peek_value();

// Then: Returns value directly, not reference
// Type is i64, not &i64
assert_eq!(value, 42);
```

### Requirement: Shared Core Implementation

Both `AeronSubscriberValueRefNode` and `AeronSubscriberValueNode` SHALL share core polling and parsing logic via a private `AeronSubscriberCore<T, F, S>` struct to avoid code duplication, differing only in their stream access trait implementations.

#### Scenario: Given both node types when implemented then share common core struct

```rust
// Internal shared core (not public API)
struct AeronSubscriberCore<T, F, S>
where
    T: Element,
    F: FnMut(&[u8]) -> Option<T>,
    S: AeronSubscriber,
{
    subscriber: S,
    parser: F,
    current_value: T,
}

impl<T, F, S> AeronSubscriberCore<T, F, S>
where
    T: Element,
    F: FnMut(&[u8]) -> Option<T>,
    S: AeronSubscriber,
{
    fn new(subscriber: S, parser: F, initial_value: T) -> Self {
        Self { subscriber, parser, current_value: initial_value }
    }

    fn poll_and_process(&mut self) -> Result<usize, TransportError> {
        self.subscriber.poll(|fragment| {
            if let Some(parsed_value) = (self.parser)(fragment) {
                self.current_value = parsed_value;
            }
            Ok(())
        })
    }

    fn current_value(&self) -> &T {
        &self.current_value
    }
}

// Public nodes wrap the core
pub struct AeronSubscriberValueRefNode<T, F, S>
where
    T: Element,
    F: FnMut(&[u8]) -> Option<T>,
    S: AeronSubscriber,
{
    core: AeronSubscriberCore<T, F, S>,
}

impl<T, F, S> AeronSubscriberValueRefNode<T, F, S> {
    pub fn new(subscriber: S, parser: F, initial_value: T) -> Self {
        Self { core: AeronSubscriberCore::new(subscriber, parser, initial_value) }
    }
}

impl<T, F, S> MutableNode for AeronSubscriberValueRefNode<T, F, S>
where
    T: Element,
    F: FnMut(&[u8]) -> Option<T> + 'static,
    S: AeronSubscriber + 'static,
{
    fn cycle(&mut self, _state: &mut GraphState) -> bool {
        let _ = self.core.poll_and_process();
        false
    }

    fn start(&mut self, state: &mut GraphState) {
        state.always_callback();
    }
}

impl<T, F, S> StreamPeekRef<T> for AeronSubscriberValueRefNode<T, F, S> {
    fn peek_ref(&self) -> &T {
        self.core.current_value()
    }
}

// ValueNode wraps same core
pub struct AeronSubscriberValueNode<T, F, S>
where
    T: Element,
    F: FnMut(&[u8]) -> Option<T>,
    S: AeronSubscriber,
{
    core: AeronSubscriberCore<T, F, S>,
}

impl<T, F, S> AeronSubscriberValueNode<T, F, S> {
    pub fn new(subscriber: S, parser: F, initial_value: T) -> Self {
        Self { core: AeronSubscriberCore::new(subscriber, parser, initial_value) }
    }
}

impl<T, F, S> MutableNode for AeronSubscriberValueNode<T, F, S>
where
    T: Element,
    F: FnMut(&[u8]) -> Option<T> + 'static,
    S: AeronSubscriber + 'static,
{
    fn cycle(&mut self, _state: &mut GraphState) -> bool {
        let _ = self.core.poll_and_process();
        false
    }

    fn start(&mut self, state: &mut GraphState) {
        state.always_callback();
    }
}

impl<T, F, S> StreamPeek<T> for AeronSubscriberValueNode<T, F, S> {
    fn peek_value(&self) -> T {
        self.core.current_value().clone()
    }

    fn peek_ref_cell(&self) -> Ref<'_, T> {
        panic!("AeronSubscriberValueNode uses value access - call peek_value() instead")
    }
}
```

### Requirement: Downstream Node Value Access Pattern

Downstream nodes SHALL be able to use `AeronSubscriberValueNode` with `StreamPeek::peek_value()` for accessing cheap-to-clone values without explicit dereferencing.

#### Scenario: Given downstream node using AeronSubscriberValueNode when accessing values then uses peek_value

```rust
struct CountingNode {
    // Upstream reference to value-based node
    upstream: Rc<RefCell<AeronSubscriberValueNode<i64, /* parser */, /* subscriber */>>>,
    count: usize,
    last_value: i64,
}

impl MutableNode for CountingNode {
    fn cycle(&mut self, _state: &mut GraphState) -> bool {
        // When: Access value using peek_value()
        let current: i64 = self.upstream.peek_value(); // Clean, no deref needed

        // Then: Can use value directly
        if current != self.last_value || self.count == 0 {
            self.count += 1;
            self.last_value = current;
        }

        false
    }

    fn start(&mut self, state: &mut GraphState) {
        state.always_callback();
    }
}
```

#### Scenario: Given downstream node when comparing access patterns then value pattern is cleaner for primitives

```rust
// Reference pattern (AeronSubscriberValueRefNode)
struct RefCountingNode<T: StreamPeekRef<i64>> {
    upstream: Rc<RefCell<T>>,
}

impl<T: StreamPeekRef<i64>> RefCountingNode<T> {
    fn get_value(&self) -> i64 {
        // Requires: borrow(), peek_ref(), deref
        *self.upstream.borrow().peek_ref()
    }
}

// Value pattern (AeronSubscriberValueNode)
struct ValueCountingNode {
    upstream: Rc<RefCell<AeronSubscriberValueNode<i64, /* */, /* */>>>,
}

impl ValueCountingNode {
    fn get_value(&self) -> i64 {
        // Cleaner: just peek_value()
        self.upstream.peek_value()
    }
}
```

### Requirement: Integration Test Demonstrates Value Pattern

An integration test SHALL demonstrate the value-access pattern using `AeronSubscriberValueNode` with a downstream counting node that uses `peek_value()` to process Aeron messages.

#### Scenario: Given integration test when using AeronSubscriberValueNode then successfully counts messages

```rust
#[test]
#[cfg(feature = "integration-tests")]
fn given_aeron_messages_when_value_node_processes_then_counts_correctly() {
    // Given: Aeron infrastructure
    let _driver = MediaDriverGuard::start().expect("Media driver");
    let subscriber = RusteronSubscriber::new(subscription);
    let mut publisher = RusteronPublisher::new(publication);

    // Publish test messages [1, 2, 3, 4, 5]
    for value in &[1i64, 2, 3, 4, 5] {
        publisher.offer(&value.to_le_bytes()).expect("Offer");
    }

    // When: Create AeronSubscriberValueNode (value-access pattern)
    let parser = |fragment: &[u8]| -> Option<i64> {
        if fragment.len() >= 8 {
            Some(i64::from_le_bytes(fragment[0..8].try_into().ok()?))
        } else {
            None
        }
    };

    let value_node = AeronSubscriberValueNode::new(subscriber, parser, 0i64);
    let value_node_rc = Rc::new(RefCell::new(value_node));

    // Create counting node using peek_value()
    let outputs = Rc::new(RefCell::new(Vec::new()));
    let outputs_clone = outputs.clone();

    struct CountingNode {
        upstream: Rc<RefCell<AeronSubscriberValueNode<i64, /* */, /* */>>>,
        count: usize,
        last_value: i64,
        output_callback: Box<dyn FnMut(usize)>,
    }

    impl MutableNode for CountingNode {
        fn cycle(&mut self, _state: &mut GraphState) -> bool {
            // Using peek_value() for clean value access
            let current = self.upstream.peek_value();

            if current != self.last_value || self.count == 0 {
                self.count += 1;
                self.last_value = current;
                (self.output_callback)(self.count);
            }

            false
        }

        fn start(&mut self, state: &mut GraphState) {
            state.always_callback();
        }
    }

    let counting_node = CountingNode {
        upstream: value_node_rc.clone(),
        count: 0,
        last_value: 0,
        output_callback: Box::new(move |count| {
            outputs_clone.borrow_mut().push(count);
        }),
    };

    // Then: Graph processes messages successfully
    let mut graph = Graph::new(
        vec![
            value_node_rc as Rc<dyn Node>,
            counting_node.into_node(),
        ],
        RunMode::RealTime,
        RunFor::Cycles(10),
    );

    graph.run().expect("Graph execution");

    // Verify counted 5 unique messages
    let final_count = outputs.borrow().last().copied().unwrap_or(0);
    assert_eq!(final_count, 5);
}
```

### Requirement: Documentation Distinguishes Node Types

Module documentation SHALL clearly explain the difference between `AeronSubscriberValueRefNode` (reference-access) and `AeronSubscriberValueNode` (value-access) with guidance on when to use each.

#### Scenario: Given module documentation when developers choose node type then clear guidance is provided

```rust
//! # Choosing Between Node Types
//!
//! ## AeronSubscriberValueRefNode<T> - Reference Access
//!
//! Implements [`StreamPeekRef<T>`](wingfoil::StreamPeekRef) for reference-based access.
//!
//! **Use when:**
//! - Type is large (>128 bytes) and expensive to clone
//! - Type is already wrapped in `Rc<T>` for sharing
//! - Implementing zero-copy patterns
//! - Need to minimize clones in hot paths
//!
//! **Access pattern:**
//! ```rust,ignore
//! let value: i64 = *self.upstream.borrow().peek_ref();
//! ```
//!
//! ## AeronSubscriberValueNode<T> - Value Access
//!
//! Implements [`StreamPeek<T>`](wingfoil::StreamPeek) for value-based access.
//!
//! **Use when:**
//! - Type is primitive (i64, f64, bool, etc.)
//! - Type implements `Copy`
//! - Type is small and cheap to clone
//! - Code clarity prioritized over micro-optimizations
//!
//! **Access pattern:**
//! ```rust,ignore
//! let value: i64 = self.upstream.peek_value(); // Clean, no deref needed
//! ```
//!
//! ## Comparison Table
//!
//! | Aspect | AeronSubscriberValueRefNode | AeronSubscriberValueNode |
//! |--------|---------------------|--------------------------|
//! | Trait | `StreamPeekRef<T>` | `StreamPeek<T>` |
//! | Access | `upstream.borrow().peek_ref()` | `upstream.peek_value()` |
//! | Returns | `&T` | `T` |
//! | Best for | Large types, Rc-wrapped | Primitives, Copy types |
//! | Cloning | Explicit via `*ref` | Implicit in return |
```

### Requirement: Choice Between Access Patterns

Downstream nodes SHALL be able to choose between reference-based access using `StreamPeekRef<T>` with `AeronSubscriberValueRefNode` or value-based access using `StreamPeek<T>` with `AeronSubscriberValueNode`, selecting the appropriate pattern based on type characteristics.

#### Scenario: Given developer choosing node type when type is primitive then uses AeronSubscriberValueNode

```rust
// For primitives: use value-access pattern
let parser = |fragment: &[u8]| -> Option<i64> { /* ... */ };
let node = AeronSubscriberValueNode::new(subscriber, parser, 0i64);
let node_rc = Rc::new(RefCell::new(node));

// Downstream uses peek_value()
struct MyNode {
    upstream: Rc<RefCell<AeronSubscriberValueNode<i64, _, _>>>,
}

impl MyNode {
    fn process(&self) {
        let value: i64 = self.upstream.peek_value(); // Clean
        // Process value...
    }
}
```

#### Scenario: Given developer choosing node type when type is large then uses AeronSubscriberValueRefNode

```rust
// For large types: use reference-access pattern
#[derive(Debug, Clone, Default)]
struct LargeMarketData {
    prices: Vec<f64>,  // Large vector
    volumes: Vec<i64>,
    // ... more fields
}

let parser = |fragment: &[u8]| -> Option<Rc<LargeMarketData>> { /* ... */ };
let node = AeronSubscriberValueRefNode::new(subscriber, parser, Rc::new(LargeMarketData::default()));
let node_rc = Rc::new(RefCell::new(node));

// Downstream uses peek_ref()
struct MarketProcessor<T: StreamPeekRef<Rc<LargeMarketData>>> {
    upstream: Rc<RefCell<T>>,
}

impl<T: StreamPeekRef<Rc<LargeMarketData>>> MarketProcessor<T> {
    fn process(&self) {
        let data_rc: Rc<LargeMarketData> = self.upstream.borrow().peek_ref().clone(); // Cheap Rc clone
        // Process data...
    }
}
```

