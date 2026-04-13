//! Wingfoil node implementations for Aeron integration.
//!
//! This module provides reusable Wingfoil nodes that bridge Aeron transport
//! with Wingfoil's stream processing framework using Element types and the
//! peek-based composition pattern.
//!
//! # Architecture Patterns
//!
//! This module supports two fundamental patterns for integrating Aeron with Wingfoil:
//!
//! ## Pattern 1: Wingfoil-Driven Polling (Traditional)
//!
//! Subscriber nodes poll Aeron during graph cycles. Best for:
//! - Simple applications with owned value types
//! - When Wingfoil controls the execution flow
//! - When Aeron idle strategies are not required
//!
//! Use [`AeronSubscriberValueNode`] or [`AeronSubscriberValueRefNode`].
//!
//! ## Pattern 2: Inverted Control (Zero-Copy)
//!
//! External Aeron polling drives Wingfoil. Best for:
//! - Zero-copy SBE decoding with flyweight lifetimes
//! - Integration with Aeron idle strategies
//! - High-frequency trading requirements
//! - When you need fine-grained control over polling
//!
//! Use [`MutableSource`] with external polling loop.
//! See `examples/inverted_control_idle_strategy.rs` for complete example.
//!
//! # Choosing Between Node Types
//!
//! This module provides three node types with different characteristics:
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
//! ## `MutableSource<T>` - External Update
//!
//! Passive state container updated from outside the graph.
//!
//! **Use when:**
//! - Need zero-copy SBE decoding with flyweight lifetimes
//! - Integrating Aeron idle strategies
//! - External event loop drives execution
//! - Processing happens in Aeron poll callbacks
//!
//! **Access pattern:** Same as value nodes - `self.upstream.peek_value()`
//!
//! See `examples/inverted_control_idle_strategy.rs` for a complete example.
//!
//! ## Comparison Table
//!
//! | Aspect | AeronSubscriberValueRefNode | AeronSubscriberValueNode | MutableSource |
//! |--------|---------------------|--------------------------|---------------|
//! | Trait | `StreamPeekRef<T>` | `StreamPeek<T>` | `StreamPeek<T>` |
//! | Access | `upstream.borrow().peek_ref()` | `upstream.peek_value()` | `upstream.peek_value()` |
//! | Returns | `&T` | `T` | `T` |
//! | Polling | Internal (cycle) | Internal (cycle) | External (user loop) |
//! | Best for | Large types, Rc-wrapped | Primitives, Copy types | Zero-copy SBE, idle strategy |
//! | Zero-copy | No (stores owned T) | No (stores owned T) | Yes (process in callback) |
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
mod publisher;
mod source;
mod subscriber;

pub use builder::AeronSubscriberNodeBuilder;
pub use publisher::{AeronPub, AeronPublisherNode, DualStreamPublisher};
pub use source::MutableSource;
pub use subscriber::{
    aeron_sub, AeronSubscriberValueNode, AeronSubscriberValueRefNode, DualStream,
};
