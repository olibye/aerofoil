//! Wingfoil node implementations for Aeron integration.
//!
//! This module provides reusable Wingfoil nodes that bridge Aeron transport
//! with Wingfoil's stream processing framework using Element types and the
//! peek-based composition pattern.
//!
//! # Choosing Between Node Types
//!
//! This module provides two Aeron subscriber node types with different access patterns:
//!
//! ## `AeronSubscriberValueRefNode<T>` - Reference Access
//!
//! Implements [`StreamPeekRef<T>`](wingfoil::StreamPeekRef) for reference-based access.
//!
//! **Use when:**
//! - Type is large (>128 bytes) and expensive to clone
//! - Type is already wrapped in `Rc<T>` for sharing
//! - Implementing zero-copy patterns
//! - Need to minimize clones in hot paths
//!
//! **Access pattern:** `*self.upstream.borrow().peek_ref()`
//!
//! See `examples/subscriber_node_reference_access.rs` for a complete example.
//!
//! ## `AeronSubscriberValueNode<T>` - Value Access
//!
//! Implements [`StreamPeek<T>`](wingfoil::StreamPeek) for value-based access.
//!
//! **Use when:**
//! - Type is primitive (i64, f64, bool, etc.)
//! - Type implements `Copy`
//! - Type is small and cheap to clone
//! - Code clarity prioritized over micro-optimizations
//!
//! **Access pattern:** `self.upstream.peek_value()` (cleaner, no deref needed)
//!
//! See `examples/subscriber_node_value_access.rs` for a complete example.
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
//!
//! # Peek-Based Composition Pattern
//!
//! Wingfoil nodes compose using peek traits:
//!
//! 1. **Transport nodes** (like [`AeronSubscriberValueRefNode`] or [`AeronSubscriberValueNode`]) poll external sources
//! 2. **Business logic nodes** accept upstream dependencies using the appropriate trait
//! 3. **Data access** uses either `peek_ref()` or `peek_value()` to read the latest value
//!
//! # Element Types
//!
//! Nodes use Wingfoil's [`Element`](wingfoil::Element) trait for message types.
//! Element requires `Debug + Clone + Default + 'static`, ensuring compatibility
//! with Wingfoil's type system. For large types, wrap them in `Rc<T>` for cheap cloning.
//!
//! # Dual-Rc Pattern for Graph Integration
//!
//! When a node needs to be both in the graph AND referenced by downstream nodes,
//! use the builder pattern which handles this automatically. The builder returns
//! `Rc<RefCell<...>>` which can be cloned for the graph and used directly for upstream.
//!
//! See `examples/dual_rc_pattern.rs` for a complete example.
//!
//! # Fan-Out Pattern
//!
//! Multiple downstream nodes can share a single upstream subscriber node.
//! Clone the `Rc<RefCell<...>>` for each downstream consumer.
//!
//! See `examples/fan_out_pattern.rs` for a complete example.

mod builder;
mod subscriber;

pub use builder::AeronSubscriberNodeBuilder;
pub use subscriber::{AeronSubscriberValueNode, AeronSubscriberValueRefNode};
