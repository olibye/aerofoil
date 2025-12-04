# Proposal: Add StreamPeek Value Strategy

## Why

Currently, Aerofoil's `AeronSubscriberNode` only implements `StreamPeekRef<T>`, which returns references (`&T`) to values. Downstream nodes must dereference these values, which requires an implicit clone:

```rust
let current_value = *self.upstream.borrow().peek_ref(); // Implicit clone via deref
```

Wingfoil provides two distinct access patterns via separate traits:
- `StreamPeekRef<T>` - returns `&T`, requires `RefCell` wrapping for graph integration
- `StreamPeek<T>` - returns `T` by value, designed for cheap-to-clone types

For cheap-to-clone Element types (primitives like `i64`, `Copy` types, small structs), implementing `StreamPeek<T>` directly on the node provides:
1. **More idiomatic API** - `peek_value()` makes the clone explicit and intentional
2. **Simpler composition** - downstream nodes don't need `RefCell::borrow()`
3. **Type-level intent** - the trait choice signals whether types are cheap to clone
4. **Cleaner code** - avoids `*ref` dereference syntax

## What Changes

Create two node types with different access strategies:

1. **`AeronSubscriberNode<T>`** (existing): Implements `StreamPeekRef<T>` for all Element types
   - Use for: Large types, `Rc<T>` wrapped types, zero-copy patterns
   - Access: `upstream.borrow().peek_ref()` returns `&T`

2. **`AeronSubscriberValueNode<T>`** (new): Implements `StreamPeek<T>` for cheap-to-clone types
   - Use for: Primitives, `Copy` types, small structs
   - Access: `upstream.peek_value()` returns `T` directly
   - No `RefCell` wrapping needed for value access
   - Still implements `MutableNode` for graph integration

### Key Differences

| Aspect | AeronSubscriberNode (Ref) | AeronSubscriberValueNode (Value) |
|--------|---------------------------|----------------------------------|
| Trait | `StreamPeekRef<T>` | `StreamPeek<T>` |
| Access | `upstream.borrow().peek_ref()` | `upstream.peek_value()` |
| Returns | `&T` | `T` |
| Graph wrapping | `Rc<RefCell<Node>>` | `Rc<Node>` |
| Upstream ref | `Rc<RefCell<Node>>` | `Rc<Node>` |
| Best for | Large types, Rc-wrapped | Primitives, Copy types |
| Cloning | Explicit via `*ref` | Implicit in return |

### Implementation Strategy

Both node types share the same core logic via a private `AeronSubscriberCore<T, F, S>` struct. The public nodes differ only in their stream access trait:

```rust
// Private shared core (not exposed in public API)
struct AeronSubscriberCore<T, F, S> {
    subscriber: S,
    parser: F,
    current_value: T,
}

impl<T, F, S> AeronSubscriberCore<T, F, S> {
    fn new(subscriber: S, parser: F, initial_value: T) -> Self { /* ... */ }
    fn poll_and_process(&mut self) -> Result<usize, TransportError> { /* ... */ }
    fn current_value(&self) -> &T { &self.current_value }
}

// Reference-access node (existing, refactored to use core)
pub struct AeronSubscriberNode<T, F, S> {
    core: AeronSubscriberCore<T, F, S>,
}

impl<T, F, S> AeronSubscriberNode<T, F, S> {
    pub fn new(subscriber: S, parser: F, initial_value: T) -> Self {
        Self { core: AeronSubscriberCore::new(subscriber, parser, initial_value) }
    }
}

impl<T, F, S> StreamPeekRef<T> for AeronSubscriberNode<T, F, S> {
    fn peek_ref(&self) -> &T {
        self.core.current_value()
    }
}

// Value-access node (new)
pub struct AeronSubscriberValueNode<T, F, S> {
    core: AeronSubscriberCore<T, F, S>,
}

impl<T, F, S> AeronSubscriberValueNode<T, F, S> {
    pub fn new(subscriber: S, parser: F, initial_value: T) -> Self {
        Self { core: AeronSubscriberCore::new(subscriber, parser, initial_value) }
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

**Benefits:**
- No macros - clearer, more maintainable code
- Explicit structure - easy to see what's shared vs node-specific
- Better IDE support - autocomplete, navigation, refactoring
- Debug friendly - real struct names in stack traces

## Sequencing

This change builds on the peek-based composition pattern established in `peek-based-composition` spec. It adds a parallel node implementation optimized for cheap-to-clone types.

## Validation

- Existing `AeronSubscriberNode` tests continue to pass
- New `AeronSubscriberValueNode` integration test demonstrates `peek_value()` usage
- Both node types tested with summing/counting downstream nodes
- Documentation examples compile for both strategies
- OpenSpec validation passes
