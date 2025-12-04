# Design: StreamPeek Value Strategy with Separate Node Implementations

## Architecture

Wingfoil provides two complementary traits for accessing node values:

```rust
// Wingfoil traits (from wingfoil-0.1.11/src/types.rs)

// Reference-based access
pub trait StreamPeekRef<T: Clone>: MutableNode {
    fn peek_ref(&self) -> &T;
}

// Value-based access
pub trait StreamPeek<T> {
    fn peek_value(&self) -> T;              // Returns by value (clone)
    fn peek_ref_cell(&self) -> Ref<'_, T>;  // Returns RefCell guard
}

// Stream combines both
pub trait Stream<T>: Node + StreamPeek<T> + AsNode {}
```

## Design Decision: Two Node Types

Instead of having one node type that implements both traits or relying on auto-implementation, we create two specialized node types that make the access pattern explicit:

### 1. AeronSubscriberNode<T> - Reference Strategy

**Implements**: `StreamPeekRef<T>`

```rust
pub struct AeronSubscriberNode<T, F, S>
where
    T: Element,
    F: FnMut(&[u8]) -> Option<T>,
    S: AeronSubscriber,
{
    subscriber: S,
    parser: F,
    current_value: T,
}

impl<T, F, S> StreamPeekRef<T> for AeronSubscriberNode<T, F, S> {
    fn peek_ref(&self) -> &T {
        &self.current_value
    }
}
```

**Graph Integration**:
```rust
let node = AeronSubscriberNode::new(subscriber, parser, 0i64);
let node_rc = Rc::new(RefCell::new(node));
let upstream_ref = node_rc.clone();        // For downstream nodes
let graph_node: Rc<dyn Node> = node_rc;    // For graph
```

**Downstream Access**:
```rust
let value: i64 = *self.upstream.borrow().peek_ref(); // Explicit deref
```

### 2. AeronSubscriberValueNode<T> - Value Strategy

**Implements**: `StreamPeek<T>` and `MutableNode` (but not `StreamPeekRef`)

```rust
pub struct AeronSubscriberValueNode<T, F, S>
where
    T: Element,
    F: FnMut(&[u8]) -> Option<T>,
    S: AeronSubscriber,
{
    subscriber: S,
    parser: F,
    current_value: T,
}

impl<T, F, S> StreamPeek<T> for AeronSubscriberValueNode<T, F, S> {
    fn peek_value(&self) -> T {
        self.current_value.clone() // Explicit clone in implementation
    }

    fn peek_ref_cell(&self) -> Ref<'_, T> {
        panic!("AeronSubscriberValueNode does not support peek_ref_cell - use peek_value instead")
    }
}
```

**Graph Integration**:
```rust
let node = AeronSubscriberValueNode::new(subscriber, parser, 0i64);
let node_rc: Rc<_> = Rc::new(node);        // No RefCell needed!
let upstream_ref = node_rc.clone();        // For downstream nodes
// For graph, would need wrapper that implements Node trait
```

**Downstream Access**:
```rust
let value: i64 = self.upstream.peek_value(); // Direct call, no borrow()
```

## Implementation Approach: Shared Core Implementation

To avoid code duplication without using macros, both node types use a shared internal struct that contains the common state and logic:

```rust
/// Internal shared implementation for Aeron subscriber nodes.
/// Not exposed in public API - use AeronSubscriberNode or AeronSubscriberValueNode instead.
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
        Self {
            subscriber,
            parser,
            current_value: initial_value,
        }
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

/// Reference-access node using StreamPeekRef<T>
pub struct AeronSubscriberNode<T, F, S>
where
    T: Element,
    F: FnMut(&[u8]) -> Option<T>,
    S: AeronSubscriber,
{
    core: AeronSubscriberCore<T, F, S>,
}

impl<T, F, S> AeronSubscriberNode<T, F, S>
where
    T: Element,
    F: FnMut(&[u8]) -> Option<T>,
    S: AeronSubscriber,
{
    pub fn new(subscriber: S, parser: F, initial_value: T) -> Self {
        Self {
            core: AeronSubscriberCore::new(subscriber, parser, initial_value),
        }
    }
}

impl<T, F, S> MutableNode for AeronSubscriberNode<T, F, S>
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

impl<T, F, S> StreamPeekRef<T> for AeronSubscriberNode<T, F, S>
where
    T: Element,
    F: FnMut(&[u8]) -> Option<T> + 'static,
    S: AeronSubscriber + 'static,
{
    fn peek_ref(&self) -> &T {
        self.core.current_value()
    }
}

/// Value-access node using StreamPeek<T>
pub struct AeronSubscriberValueNode<T, F, S>
where
    T: Element,
    F: FnMut(&[u8]) -> Option<T>,
    S: AeronSubscriber,
{
    core: AeronSubscriberCore<T, F, S>,
}

impl<T, F, S> AeronSubscriberValueNode<T, F, S>
where
    T: Element,
    F: FnMut(&[u8]) -> Option<T>,
    S: AeronSubscriber,
{
    pub fn new(subscriber: S, parser: F, initial_value: T) -> Self {
        Self {
            core: AeronSubscriberCore::new(subscriber, parser, initial_value),
        }
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

impl<T, F, S> StreamPeek<T> for AeronSubscriberValueNode<T, F, S>
where
    T: Element,
    F: FnMut(&[u8]) -> Option<T> + 'static,
    S: AeronSubscriber + 'static,
{
    fn peek_value(&self) -> T {
        self.core.current_value().clone()
    }

    fn peek_ref_cell(&self) -> Ref<'_, T> {
        panic!("AeronSubscriberValueNode uses value access - call peek_value() instead")
    }
}
```

### Benefits of Core Struct Approach

1. **No Macros**: Clearer, more maintainable code
2. **Explicit Structure**: Easy to see what's shared vs node-specific
3. **Type Safety**: Rust's type system validates everything
4. **Debug Friendly**: Stack traces show real struct names, not macro expansions
5. **IDE Support**: Better autocomplete and navigation

## Graph Integration Challenge

**Issue**: Wingfoil's `Node` trait requires `RefCell` wrapper, but `StreamPeek` doesn't need it for value access.

**Solutions**:

### Option A: Always Use RefCell (Simpler)

```rust
// Even for value node, wrap in RefCell for graph
let node = AeronSubscriberValueNode::new(subscriber, parser, 0i64);
let node_rc = Rc::new(RefCell::new(node));
let graph_node: Rc<dyn Node> = node_rc;

// Downstream still benefits from peek_value()
let value = upstream.peek_value(); // Works on RefCell<ValueNode>
```

**Pro**: Consistent graph integration pattern
**Con**: Still requires RefCell even though we're not using references

### Option B: Direct Node Implementation (More Complex)

Implement `Node` directly on `AeronSubscriberValueNode` without `RefCell`:

```rust
impl<T, F, S> Node for AeronSubscriberValueNode<T, F, S> {
    fn cycle(&self, state: &mut GraphState) -> bool {
        // Problem: cycle() needs &mut self to poll, but Node::cycle takes &self
        // Would need internal mutability via RefCell anyway!
    }
}
```

**Problem**: Can't avoid `RefCell` because polling requires mutation.

### Recommendation: Option A with Documentation

Accept that `RefCell` is needed for graph integration due to mutation requirements. The value comes from:
1. **Cleaner downstream code**: `upstream.peek_value()` vs `*upstream.borrow().peek_ref()`
2. **Type-level intent**: `StreamPeek<i64>` signals cheap-to-clone
3. **Less indirection**: One less dereference in downstream nodes

## When to Use Each Node Type

### Use AeronSubscriberNode (Ref) when:
- Type is large (>128 bytes)
- Type is already `Rc<T>` wrapped
- Implementing zero-copy patterns
- Need to avoid clones in hot path
- Working with non-Copy types

### Use AeronSubscriberValueNode (Value) when:
- Type is primitive (i64, f64, bool, etc.)
- Type is Copy
- Type is small (<= 128 bytes) and cheap to clone
- Code clarity prioritized over micro-optimizations
- Prefer explicit value semantics

## Performance Considerations

### RefCell Overhead

Both node types require `RefCell` for graph integration. The difference is in downstream access:

**AeronSubscriberNode (Ref)**:
```rust
let value: i64 = *self.upstream.borrow().peek_ref();
// 1. RefCell::borrow() - runtime borrow check
// 2. peek_ref() - returns &i64
// 3. Deref (*) - copies i64
```

**AeronSubscriberValueNode (Value)**:
```rust
let value: i64 = self.upstream.peek_value();
// 1. StreamPeek::peek_value() on RefCell (if auto-impl)
// 2. RefCell::borrow() inside peek_value
// 3. Clone the value
```

Wait - if we use RefCell wrapper, we get auto-implementation of `StreamPeek`! Let me check Wingfoil again...

Looking at Wingfoil types.rs:136-147, `StreamPeek` is auto-implemented for `RefCell<STREAM>` where `STREAM: StreamPeekRef`. This means:

**Key Insight**: If `AeronSubscriberValueNode` implements `StreamPeekRef`, it automatically gets `StreamPeek` via RefCell wrapper!

## Revised Design: Single Node, Clear Trait Boundaries

**Better approach**: Keep `AeronSubscriberNode` with `StreamPeekRef`, and let `RefCell` provide both access patterns:

```rust
let node_rc = Rc::new(RefCell::new(AeronSubscriberNode::new(...)));

// Reference access (when wrapped in RefCell, bypass auto-impl)
let value = *node_rc.borrow().peek_ref();

// Value access (auto-implemented by Wingfoil)
let value = node_rc.peek_value(); // Calls RefCell's impl of StreamPeek
```

But user wants **separate node types**, so let's stick with the original plan but be clear about the trade-offs.

## Final Recommendation

Implement `AeronSubscriberValueNode` as a distinct type that:
1. Implements `StreamPeek<T>` directly (not StreamPeekRef)
2. Still requires `RefCell` for graph integration
3. Makes the value-access pattern explicit and intentional
4. Provides better API ergonomics for cheap-to-clone types

The performance will be similar, but the type-level distinction makes the intent clear and provides better ergonomics for downstream nodes working with primitives.
