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
//! ## AeronSubscriberNode<T> - Reference Access
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
//! | Aspect | AeronSubscriberNode | AeronSubscriberValueNode |
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
//! 1. **Transport nodes** (like [`AeronSubscriberNode`] or [`AeronSubscriberValueNode`]) poll external sources
//! 2. **Business logic nodes** accept upstream dependencies using the appropriate trait
//! 3. **Data access** uses either `peek_ref()` or `peek_value()` to read the latest value
//!
//! # Element Types
//!
//! Nodes use Wingfoil's [`Element`](wingfoil::Element) trait for message types.
//! Element requires `Debug + Clone + Default + 'static`, ensuring compatibility
//! with Wingfoil's type system. For large types, wrap them in `Rc<T>` for cheap cloning.
//!
//! # Example: Dual-Rc Pattern for Graph Integration
//!
//! When a node needs to be both in the graph AND referenced by downstream nodes:
//!
//! ```rust,ignore
//! use std::cell::RefCell;
//! use std::rc::Rc;
//! use wingfoil::Node;
//!
//! // Create node
//! let subscriber_node = AeronSubscriberNode::new(subscriber, parser, 0i64);
//!
//! // Wrap in Rc<RefCell<>> manually (don't use into_node() yet)
//! let subscriber_rc = Rc::new(RefCell::new(subscriber_node));
//!
//! // Clone for upstream reference (keeps concrete type for peek access)
//! let upstream_ref = subscriber_rc.clone();
//!
//! // Cast to Rc<dyn Node> for graph (type erasure for graph vector)
//! let graph_node: Rc<dyn Node> = subscriber_rc;
//!
//! // Create downstream node with upstream reference
//! let business_logic_node = MyNode::new(upstream_ref, /*...*/);
//!
//! // Add both to graph
//! let graph = Graph::new(
//!     vec![
//!         graph_node,                      // Transport node as dyn Node
//!         business_logic_node.into_node(), // Business logic as dyn Node
//!     ],
//!     RunMode::RealTime,
//!     RunFor::Cycles(100)
//! );
//! ```
//!
//! This "dual-Rc" pattern is necessary because:
//! - Downstream nodes need the concrete type to call `peek_ref()`
//! - The graph needs `Rc<dyn Node>` for its heterogeneous vector
//! - Wingfoil's `into_node()` consumes the value, so we manage Rc ourselves

mod subscriber;

pub use subscriber::{AeronSubscriberNode, AeronSubscriberValueNode};
