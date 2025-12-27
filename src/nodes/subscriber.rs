//! Aeron subscriber node for Wingfoil stream processing.
//!
//! This module provides [`AeronSubscriberValueRefNode`] and [`AeronSubscriberValueNode`],
//! Wingfoil nodes that bridge Aeron transport with Wingfoil's stream processing
//! framework using Element types.

use crate::transport::{AeronSubscriber, TransportError};
use wingfoil::{Element, GraphState, MutableNode, StreamPeekRef, UpStreams};

/// Internal shared implementation for Aeron subscriber nodes.
///
/// This struct contains the common state and logic used by both [`AeronSubscriberValueRefNode`]
/// and [`AeronSubscriberValueNode`]. It is not part of the public API - users should
/// use one of the public node types instead.
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

/// A Wingfoil node that polls an Aeron subscriber and implements `StreamPeekRef<T>`.
///
/// This node bridges Aeron transport with Wingfoil's stream processing by:
/// - Polling an [`AeronSubscriber`] for incoming messages (non-blocking)
/// - Parsing messages using a user-provided parser function
/// - Storing the latest parsed value for downstream consumption via `peek_ref()`
///
/// # Type Parameters
///
/// - `T`: The type of values produced by parsing Aeron messages (must implement `Element`)
/// - `F`: The parser function type, `FnMut(&[u8]) -> Option<T>`
/// - `S`: The Aeron subscriber implementation
///
/// # Element Trait
///
/// The message type `T` must implement Wingfoil's `Element` trait, which requires:
/// `Debug + Clone + Default + 'static`. This ensures:
/// - `Debug`: For logging and debugging
/// - `Clone`: For value copying (must be cheap to clone - use `Rc<T>` for large types)
/// - `Default`: For providing an initial value
/// - `'static`: No non-static references
///
/// This ensures compatibility with Wingfoil's type system and enables use with
/// standard Wingfoil stream operators.
///
/// # Parser Function Contract
///
/// The parser function receives a byte slice (`&[u8]`) containing a message fragment
/// and returns `Option<T>`:
/// - `Some(value)` - Message was successfully parsed, updates current value
/// - `None` - Message was invalid/incomplete, current value unchanged
///
/// # StreamPeekRef Implementation
///
/// This node implements `StreamPeekRef<T>`, allowing downstream nodes to access
/// the latest parsed value via `peek_ref()`. This follows Wingfoil's idiomatic
/// pattern for node composition.
///
/// # Single-threaded Design
///
/// Following Wingfoil's design, this node is designed for single-threaded execution
/// and uses simple state management without synchronization primitives.
///
/// # Example
///
/// See `examples/subscriber_node_reference_access.rs` for a complete runnable example.
pub struct AeronSubscriberValueRefNode<T, F, S>
where
    T: Element,
    F: FnMut(&[u8]) -> Option<T>,
    S: AeronSubscriber,
{
    core: AeronSubscriberCore<T, F, S>,
}

impl<T, F, S> AeronSubscriberValueRefNode<T, F, S>
where
    T: Element,
    F: FnMut(&[u8]) -> Option<T>,
    S: AeronSubscriber,
{
    /// Creates a new `AeronSubscriberValueRefNode`.
    ///
    /// # Parameters
    ///
    /// - `subscriber`: The Aeron subscriber to poll for messages
    /// - `parser`: Function to parse byte fragments into typed values
    /// - `initial_value`: The initial value before any messages are received
    ///
    /// # Returns
    ///
    /// A new `AeronSubscriberValueRefNode` instance ready to be added to a Wingfoil graph.
    ///
    /// See `examples/subscriber_node_reference_access.rs` for usage.
    pub fn new(subscriber: S, parser: F, initial_value: T) -> Self {
        Self {
            core: AeronSubscriberCore::new(subscriber, parser, initial_value),
        }
    }

    /// Creates a new builder for constructing this node type.
    ///
    /// Returns `Rc<RefCell<Self>>` which can be cloned for the graph
    /// and used directly as upstream reference.
    ///
    /// See `examples/dual_rc_pattern.rs` for usage.
    pub fn builder() -> super::builder::AeronSubscriberNodeBuilder<T, F, S>
    where
        F: 'static,
        S: 'static,
    {
        super::builder::AeronSubscriberNodeBuilder::new()
    }
}

/// Wingfoil `MutableNode` implementation.
///
/// This enables the node to be registered in a Wingfoil graph and receive
/// automatic cycle callbacks for polling and processing messages.
impl<T, F, S> MutableNode for AeronSubscriberValueRefNode<T, F, S>
where
    T: Element,
    F: FnMut(&[u8]) -> Option<T> + 'static,
    S: AeronSubscriber + 'static,
{
    /// Called by Wingfoil on each graph cycle to poll for and process messages.
    ///
    /// This method polls the Aeron subscriber (non-blocking) and processes any
    /// available messages, updating the current value when messages are successfully
    /// parsed. Returns `false` to indicate the node should continue processing.
    fn cycle(&mut self, _state: &mut GraphState) -> anyhow::Result<bool> {
        // Poll and process any available messages
        // Ignore errors - we continue processing on the next cycle
        let _ = self.core.poll_and_process();

        // Return false to indicate we want to continue processing
        // (the graph will control when to stop based on its run configuration)
        Ok(false)
    }

    /// Register this node to be called on every cycle.
    ///
    /// This ensures the node continuously polls for incoming messages
    /// throughout the graph's execution.
    fn start(&mut self, state: &mut GraphState) -> anyhow::Result<()> {
        state.always_callback();
        Ok(())
    }
    
    fn upstreams(&self) -> wingfoil::UpStreams {
        UpStreams::none()
    }
}

/// Wingfoil `StreamPeekRef<T>` implementation.
///
/// This allows downstream nodes to access the latest parsed value via `peek_ref()`,
/// enabling Wingfoil's idiomatic node composition pattern.
impl<T, F, S> StreamPeekRef<T> for AeronSubscriberValueRefNode<T, F, S>
where
    T: Element,
    F: FnMut(&[u8]) -> Option<T> + 'static,
    S: AeronSubscriber + 'static,
{
    /// Returns a reference to the most recently parsed value.
    ///
    /// Downstream nodes can call this method to access the latest value
    /// produced by this stream. If no messages have been successfully parsed,
    /// this returns a reference to the initial value provided during construction.
    fn peek_ref(&self) -> &T {
        self.core.current_value()
    }
}

/// A Wingfoil node that polls an Aeron subscriber and implements `StreamPeek<T>`.
///
/// This node provides value-based access for cheap-to-clone types, complementing
/// [`AeronSubscriberValueRefNode`] which uses reference-based access. This node bridges
/// Aeron transport with Wingfoil's stream processing by:
/// - Polling an [`AeronSubscriber`] for incoming messages (non-blocking)
/// - Parsing messages using a user-provided parser function
/// - Providing the latest parsed value via `peek_value()` for downstream consumption
///
/// # Type Parameters
///
/// - `T`: The type of values produced by parsing Aeron messages (must implement `Element`)
/// - `F`: The parser function type, `FnMut(&[u8]) -> Option<T>`
/// - `S`: The Aeron subscriber implementation
///
/// # Element Trait
///
/// The message type `T` must implement Wingfoil's `Element` trait, which requires:
/// `Debug + Clone + Default + 'static`. This ensures:
/// - `Debug`: For logging and debugging
/// - `Clone`: For value copying (must be cheap to clone - use `AeronSubscriberValueRefNode` for large types)
/// - `Default`: For providing an initial value
/// - `'static`: No non-static references
///
/// # StreamPeek Implementation
///
/// This node implements `StreamPeek<T>`, allowing downstream nodes to access
/// the latest parsed value via `peek_value()`. This provides cleaner ergonomics
/// for cheap-to-clone types compared to the reference-based pattern.
///
/// # Choosing Between Node Types
///
/// - **Use `AeronSubscriberValueNode`** (this type) for:
///   - Primitives (i64, f64, bool, etc.)
///   - Types implementing `Copy`
///   - Small structs (<= 128 bytes)
///   - When code clarity is prioritized
///
/// - **Use `AeronSubscriberValueRefNode`** for:
///   - Large types (> 128 bytes)
///   - `Rc<T>` wrapped types
///   - Zero-copy patterns
///   - Performance-critical hot paths
///
/// # Example
///
/// See `examples/subscriber_node_value_access.rs` for a complete runnable example.
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
    /// Creates a new `AeronSubscriberValueNode`.
    ///
    /// # Parameters
    ///
    /// - `subscriber`: The Aeron subscriber to poll for messages
    /// - `parser`: Function to parse byte fragments into typed values
    /// - `initial_value`: The initial value before any messages are received
    ///
    /// # Returns
    ///
    /// A new `AeronSubscriberValueNode` instance ready to be added to a Wingfoil graph.
    ///
    /// # Example
    ///
    /// See `examples/subscriber_node_value_access.rs` for a complete example.
    pub fn new(subscriber: S, parser: F, initial_value: T) -> Self {
        Self {
            core: AeronSubscriberCore::new(subscriber, parser, initial_value),
        }
    }

    /// Creates a new builder for constructing this node type.
    ///
    /// Returns `Rc<RefCell<Self>>` which can be cloned for the graph
    /// and used directly as upstream reference.
    ///
    /// # Example
    ///
    /// See `examples/subscriber_node_value_access.rs` for a complete example.
    pub fn builder() -> super::builder::AeronSubscriberNodeBuilder<T, F, S>
    where
        F: 'static,
        S: 'static,
    {
        super::builder::AeronSubscriberNodeBuilder::new()
    }
}

/// Wingfoil `MutableNode` implementation.
///
/// This enables the node to be registered in a Wingfoil graph and receive
/// automatic cycle callbacks for polling and processing messages.
impl<T, F, S> MutableNode for AeronSubscriberValueNode<T, F, S>
where
    T: Element,
    F: FnMut(&[u8]) -> Option<T> + 'static,
    S: AeronSubscriber + 'static,
{
    /// Called by Wingfoil on each graph cycle to poll for and process messages.
    ///
    /// This method polls the Aeron subscriber (non-blocking) and processes any
    /// available messages, updating the current value when messages are successfully
    /// parsed. Returns `false` to indicate the node should continue processing.
    fn cycle(&mut self, _state: &mut GraphState) -> anyhow::Result<bool> {
        // Poll and process any available messages
        // Ignore errors - we continue processing on the next cycle
        let _ = self.core.poll_and_process();

        // Return false to indicate we want to continue processing
        // (the graph will control when to stop based on its run configuration)
        Ok(false)
    }

    /// Register this node to be called on every cycle.
    ///
    /// This ensures the node continuously polls for incoming messages
    /// throughout the graph's execution.
    fn start(&mut self, state: &mut GraphState)-> anyhow::Result<()>{
        state.always_callback();
        Ok(())
    }
    
    fn upstreams(&self) -> UpStreams {
        UpStreams::none()
    }
}

/// Wingfoil `StreamPeekRef<T>` implementation.
///
/// This allows `AeronSubscriberValueNode` to work with Wingfoil's auto-implementation
/// of `StreamPeek` for `RefCell<T>`, enabling the value-access pattern when wrapped
/// in RefCell for graph integration.
impl<T, F, S> StreamPeekRef<T> for AeronSubscriberValueNode<T, F, S>
where
    T: Element,
    F: FnMut(&[u8]) -> Option<T> + 'static,
    S: AeronSubscriber + 'static,
{
    /// Returns a reference to the most recently parsed value.
    ///
    /// This implementation enables Wingfoil's auto-implementation of `StreamPeek`
    /// for `RefCell<AeronSubscriberValueNode>`, which provides the `peek_value()`
    /// method for value-based access.
    fn peek_ref(&self) -> &T {
        self.core.current_value()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::{FragmentBuffer, FragmentHeader};
    use std::cell::RefCell;

    /// Mock subscriber for testing that provides canned message sequences
    struct MockSubscriber {
        messages: RefCell<Vec<Vec<u8>>>,
    }

    impl MockSubscriber {
        fn new(messages: Vec<Vec<u8>>) -> Self {
            MockSubscriber {
                messages: RefCell::new(messages),
            }
        }
    }

    impl AeronSubscriber for MockSubscriber {
        fn poll<F>(&mut self, mut handler: F) -> Result<usize, TransportError>
        where
            F: FnMut(&crate::transport::FragmentBuffer) -> Result<(), TransportError>,
        {
            let mut messages = self.messages.borrow_mut();
            let count = messages.len();

            // Process all available messages
            for message in messages.drain(..) {
                // Create a FragmentBuffer with dummy header metadata
                let header = FragmentHeader {
                    position: 0,
                    session_id: 0,
                    stream_id: 0,
                };
                let fragment = FragmentBuffer::new(&message, header);
                handler(&fragment)?;
            }

            Ok(count)
        }
    }

    /// Test: Given valid i64 messages, when polled, then updates current_value
    #[test]
    fn given_valid_messages_when_polled_then_updates_current_value() {
        // Given: Mock subscriber with two i64 messages (little-endian)
        let msg1 = 42i64.to_le_bytes().to_vec();
        let msg2 = 100i64.to_le_bytes().to_vec();
        let subscriber = MockSubscriber::new(vec![msg1, msg2]);

        // Given: Parser for i64 messages
        let parser = |fragment: &[u8]| -> Option<i64> {
            if fragment.len() >= 8 {
                let bytes: [u8; 8] = fragment[0..8].try_into().ok()?;
                Some(i64::from_le_bytes(bytes))
            } else {
                None
            }
        };

        // Given: AeronSubscriberValueRefNode with initial value 0
        let mut node = AeronSubscriberValueRefNode::new(subscriber, parser, 0);

        // When: Poll and process messages
        let result = node.core.poll_and_process();

        // Then: Polling succeeds and processes 2 messages
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);

        // Then: Current value is updated to the last message (100)
        assert_eq!(*node.peek_ref(), 100);
    }

    /// Test: Given custom Element type, when used with AeronSubscriberValueRefNode, then compiles and works
    #[test]
    fn given_custom_element_type_when_used_then_works() {
        #[derive(Debug, Clone, Default, PartialEq)]
        struct Trade {
            price: f64,
            quantity: i64,
        }
        // Element is automatically implemented for types that are Debug + Clone + Default + 'static

        // Given: Parser for Trade messages
        let parser = |fragment: &[u8]| -> Option<Trade> {
            if fragment.len() >= 16 {
                let price = f64::from_le_bytes(fragment[0..8].try_into().ok()?);
                let quantity = i64::from_le_bytes(fragment[8..16].try_into().ok()?);
                Some(Trade { price, quantity })
            } else {
                None
            }
        };

        // Given: Mock subscriber with Trade message
        let mut msg = Vec::new();
        msg.extend_from_slice(&123.45f64.to_le_bytes());
        msg.extend_from_slice(&100i64.to_le_bytes());
        let subscriber = MockSubscriber::new(vec![msg]);

        // When: Create node with Element type
        let mut node = AeronSubscriberValueRefNode::new(subscriber, parser, Trade::default());

        // Then: Compiles successfully and parses correctly
        node.core.poll_and_process().unwrap();
        assert_eq!(node.peek_ref().price, 123.45);
        assert_eq!(node.peek_ref().quantity, 100);
    }

    /// Test: Given peek_ref when called then returns latest parsed value
    #[test]
    fn given_peek_ref_when_called_then_returns_latest_parsed_value() {
        // Given: Mock subscriber with one i64 message
        let msg = 999i64.to_le_bytes().to_vec();
        let subscriber = MockSubscriber::new(vec![msg]);

        let parser = |fragment: &[u8]| -> Option<i64> {
            if fragment.len() >= 8 {
                let bytes: [u8; 8] = fragment[0..8].try_into().ok()?;
                Some(i64::from_le_bytes(bytes))
            } else {
                None
            }
        };

        let mut node = AeronSubscriberValueRefNode::new(subscriber, parser, 0);

        // When: Poll and process messages
        node.core.poll_and_process().unwrap();

        // Then: peek_ref returns the parsed value
        assert_eq!(*node.peek_ref(), 999);
    }

    /// Test: Given invalid messages when polled then keeps previous value
    #[test]
    fn given_invalid_messages_when_polled_then_keeps_previous_value() {
        // Given: Mock subscriber with invalid short message (only 4 bytes, need 8)
        let invalid_msg = vec![1, 2, 3, 4];
        let subscriber = MockSubscriber::new(vec![invalid_msg]);

        let parser = |fragment: &[u8]| -> Option<i64> {
            if fragment.len() >= 8 {
                let bytes: [u8; 8] = fragment[0..8].try_into().ok()?;
                Some(i64::from_le_bytes(bytes))
            } else {
                None
            }
        };

        // Given: Initial value of 42
        let mut node = AeronSubscriberValueRefNode::new(subscriber, parser, 42);

        // When: Poll and process invalid message
        node.core.poll_and_process().unwrap();

        // Then: Current value remains at initial value (42)
        assert_eq!(*node.peek_ref(), 42);
    }

    /// Test: Given no messages when polled then keeps previous value
    #[test]
    fn given_no_messages_when_polled_then_keeps_previous_value() {
        // Given: Mock subscriber with no messages
        let subscriber = MockSubscriber::new(vec![]);

        let parser = |fragment: &[u8]| -> Option<i64> {
            if fragment.len() >= 8 {
                let bytes: [u8; 8] = fragment[0..8].try_into().ok()?;
                Some(i64::from_le_bytes(bytes))
            } else {
                None
            }
        };

        // Given: Initial value of 123
        let mut node = AeronSubscriberValueRefNode::new(subscriber, parser, 123);

        // When: Poll with no messages available
        let result = node.core.poll_and_process();

        // Then: Polling succeeds with 0 messages processed
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);

        // Then: Current value remains at initial value (123)
        assert_eq!(*node.peek_ref(), 123);
    }
}
