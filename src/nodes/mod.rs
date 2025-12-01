//! Wingfoil node implementations for Aeron integration.
//!
//! This module provides reusable Wingfoil nodes that bridge Aeron transport
//! with Wingfoil's stream processing framework using Element types and the
//! peek-based composition pattern.
//!
//! # Peek-Based Composition Pattern
//!
//! Wingfoil nodes compose using `StreamPeekRef<T>` trait and the peek pattern:
//!
//! 1. **Transport nodes** (like [`AeronSubscriberNode`]) poll external sources and implement `StreamPeekRef<T>`
//! 2. **Business logic nodes** accept upstream dependencies as `Rc<RefCell<dyn StreamPeekRef<T>>>`
//! 3. **Data access** uses `upstream.borrow().peek_ref()` to read the latest value
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

pub use subscriber::AeronSubscriberNode;
