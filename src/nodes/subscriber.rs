//! Aeron subscriber node for Wingfoil stream processing.
//!
//! This module provides [`AeronSubscriberValueRefNode`] and [`AeronSubscriberValueNode`],
//! Wingfoil nodes that bridge Aeron transport with Wingfoil's stream processing
//! framework using Element types.

use crate::nodes::MutableSource;
use crate::transport::{AeronStatus, AeronSubscriber, TransportError};
use std::cell::RefCell;
use std::rc::Rc;
use wingfoil::{Element, GraphState, MutableNode, StreamPeekRef, UpStreams};

/// Internal shared implementation for Aeron subscriber nodes.
///
/// This struct contains the common state and logic used by both [`AeronSubscriberValueRefNode`]
/// and [`AeronSubscriberValueNode`]. It is not part of the public API - users should
/// use one of the public node types instead.
struct AeronSubscriberCore<T, F, S>
where
    T: Element,
    F: FnMut(&[u8]) -> Result<Option<T>, TransportError>,
    S: AeronSubscriber,
{
    subscriber: S,
    parser: F,
    current_value: T,
    status: Option<Rc<RefCell<MutableSource<AeronStatus>>>>,
    last_connected: bool,
}

impl<T, F, S> AeronSubscriberCore<T, F, S>
where
    T: Element,
    F: FnMut(&[u8]) -> Result<Option<T>, TransportError>,
    S: AeronSubscriber,
{
    fn new(subscriber: S, parser: F, initial_value: T) -> Self {
        Self {
            subscriber,
            parser,
            current_value: initial_value,
            status: None,
            last_connected: false,
        }
    }

    fn with_status(
        subscriber: S,
        parser: F,
        initial_value: T,
        status: Rc<RefCell<MutableSource<AeronStatus>>>,
    ) -> Self {
        Self {
            subscriber,
            parser,
            current_value: initial_value,
            status: Some(status),
            last_connected: false,
        }
    }

    fn poll_and_process(&mut self) -> Result<usize, TransportError> {
        let poll_result = self.subscriber.poll(|fragment| {
            match (self.parser)(fragment) {
                Ok(Some(parsed_value)) => {
                    self.current_value = parsed_value;
                }
                Ok(None) => {}
                Err(e) => return Err(e),
            }
            Ok(())
        });

        if let Some(status) = &self.status {
            let connected = self.subscriber.is_connected();
            if connected != self.last_connected {
                status.borrow_mut().set(if connected {
                    AeronStatus::Connected
                } else {
                    AeronStatus::Disconnected
                });
                self.last_connected = connected;
            }
        }

        poll_result
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
/// - `F`: The parser function type, `FnMut(&[u8]) -> Result<Option<T>, TransportError>`
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
/// and returns `Result<Option<T>, TransportError>`:
/// - `Ok(Some(value))` - Message was successfully parsed, updates current value
/// - `Ok(None)` - Message was invalid/incomplete, current value unchanged
/// - `Err(e)` - Parse error, propagated to the graph runner
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
    F: FnMut(&[u8]) -> Result<Option<T>, TransportError>,
    S: AeronSubscriber,
{
    core: AeronSubscriberCore<T, F, S>,
}

impl<T, F, S> AeronSubscriberValueRefNode<T, F, S>
where
    T: Element,
    F: FnMut(&[u8]) -> Result<Option<T>, TransportError>,
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
    F: FnMut(&[u8]) -> Result<Option<T>, TransportError> + 'static,
    S: AeronSubscriber + 'static,
{
    /// Called by Wingfoil on each graph cycle to poll for and process messages.
    ///
    /// This method polls the Aeron subscriber (non-blocking) and processes any
    /// available messages, updating the current value when messages are successfully
    /// parsed. Returns `false` to indicate the node should continue processing.
    fn cycle(&mut self, _state: &mut GraphState) -> anyhow::Result<bool> {
        self.core.poll_and_process()?;
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
    F: FnMut(&[u8]) -> Result<Option<T>, TransportError> + 'static,
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
/// - `F`: The parser function type, `FnMut(&[u8]) -> Result<Option<T>, TransportError>`
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
    F: FnMut(&[u8]) -> Result<Option<T>, TransportError>,
    S: AeronSubscriber,
{
    core: AeronSubscriberCore<T, F, S>,
}

impl<T, F, S> AeronSubscriberValueNode<T, F, S>
where
    T: Element,
    F: FnMut(&[u8]) -> Result<Option<T>, TransportError>,
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
    F: FnMut(&[u8]) -> Result<Option<T>, TransportError> + 'static,
    S: AeronSubscriber + 'static,
{
    /// Called by Wingfoil on each graph cycle to poll for and process messages.
    ///
    /// This method polls the Aeron subscriber (non-blocking) and processes any
    /// available messages, updating the current value when messages are successfully
    /// parsed. Returns `false` to indicate the node should continue processing.
    fn cycle(&mut self, _state: &mut GraphState) -> anyhow::Result<bool> {
        self.core.poll_and_process()?;
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
    F: FnMut(&[u8]) -> Result<Option<T>, TransportError> + 'static,
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

/// Return type for [`aeron_sub`]: `(data_node, status_source)`.
pub type DualStream<T, F, S> = (
    Rc<RefCell<AeronSubscriberValueRefNode<T, F, S>>>,
    Rc<RefCell<MutableSource<AeronStatus>>>,
);

/// Creates a dual-stream Aeron subscriber returning data and status streams.
///
/// This mirrors Wingfoil's `zmq_sub<T>()` pattern: a single function call
/// returns both a data node (for parsed messages) and a status source (for
/// connection state transitions). The data node checks `is_connected()` each
/// cycle and updates the status source only on transitions.
pub fn aeron_sub<T, F, S>(subscriber: S, parser: F, initial_value: T) -> DualStream<T, F, S>
where
    T: Element,
    F: FnMut(&[u8]) -> Result<Option<T>, TransportError> + 'static,
    S: AeronSubscriber + 'static,
{
    let status = Rc::new(RefCell::new(MutableSource::new(AeronStatus::Disconnected)));
    let node = AeronSubscriberValueRefNode {
        core: AeronSubscriberCore::with_status(
            subscriber,
            parser,
            initial_value,
            Rc::clone(&status),
        ),
    };
    (Rc::new(RefCell::new(node)), status)
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

    struct ConnectedMockSubscriber {
        messages: Vec<Vec<u8>>,
        connected: bool,
    }

    impl ConnectedMockSubscriber {
        fn new(messages: Vec<Vec<u8>>, connected: bool) -> Self {
            Self {
                messages,
                connected,
            }
        }
    }

    impl AeronSubscriber for ConnectedMockSubscriber {
        fn poll<H>(&mut self, mut handler: H) -> Result<usize, TransportError>
        where
            H: FnMut(&FragmentBuffer) -> Result<(), TransportError>,
        {
            let count = self.messages.len();
            for message in self.messages.drain(..) {
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

        fn is_connected(&self) -> bool {
            self.connected
        }
    }

    fn fallible_i64_parser(fragment: &[u8]) -> Result<Option<i64>, TransportError> {
        if fragment.len() >= 8 {
            let bytes: [u8; 8] = fragment[0..8].try_into().unwrap();
            Ok(Some(i64::from_le_bytes(bytes)))
        } else {
            Ok(None)
        }
    }

    /// Test: Given valid i64 messages, when polled, then updates current_value
    #[test]
    fn given_valid_messages_when_polled_then_updates_current_value() {
        let msg1 = 42i64.to_le_bytes().to_vec();
        let msg2 = 100i64.to_le_bytes().to_vec();
        let subscriber = MockSubscriber::new(vec![msg1, msg2]);

        let mut node = AeronSubscriberValueRefNode::new(subscriber, fallible_i64_parser, 0);

        let result = node.core.poll_and_process();

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 2);
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

        let parser = |fragment: &[u8]| -> Result<Option<Trade>, TransportError> {
            if fragment.len() >= 16 {
                let price = f64::from_le_bytes(fragment[0..8].try_into().unwrap());
                let quantity = i64::from_le_bytes(fragment[8..16].try_into().unwrap());
                Ok(Some(Trade { price, quantity }))
            } else {
                Ok(None)
            }
        };

        let mut msg = Vec::new();
        msg.extend_from_slice(&123.45f64.to_le_bytes());
        msg.extend_from_slice(&100i64.to_le_bytes());
        let subscriber = MockSubscriber::new(vec![msg]);

        let mut node = AeronSubscriberValueRefNode::new(subscriber, parser, Trade::default());

        node.core.poll_and_process().unwrap();
        assert_eq!(node.peek_ref().price, 123.45);
        assert_eq!(node.peek_ref().quantity, 100);
    }

    #[test]
    fn given_peek_ref_when_called_then_returns_latest_parsed_value() {
        let msg = 999i64.to_le_bytes().to_vec();
        let subscriber = MockSubscriber::new(vec![msg]);

        let mut node = AeronSubscriberValueRefNode::new(subscriber, fallible_i64_parser, 0);

        node.core.poll_and_process().unwrap();
        assert_eq!(*node.peek_ref(), 999);
    }

    #[test]
    fn given_invalid_messages_when_polled_then_keeps_previous_value() {
        let invalid_msg = vec![1, 2, 3, 4];
        let subscriber = MockSubscriber::new(vec![invalid_msg]);

        let mut node = AeronSubscriberValueRefNode::new(subscriber, fallible_i64_parser, 42);

        node.core.poll_and_process().unwrap();
        assert_eq!(*node.peek_ref(), 42);
    }

    #[test]
    fn given_no_messages_when_polled_then_keeps_previous_value() {
        let subscriber = MockSubscriber::new(vec![]);

        let mut node = AeronSubscriberValueRefNode::new(subscriber, fallible_i64_parser, 123);

        let result = node.core.poll_and_process();

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
        assert_eq!(*node.peek_ref(), 123);
    }

    #[test]
    fn given_parser_returning_err_when_cycle_then_error_propagated() {
        let msg = 42i64.to_le_bytes().to_vec();
        let subscriber = MockSubscriber::new(vec![msg]);

        let error_parser = |_fragment: &[u8]| -> Result<Option<i64>, TransportError> {
            Err(TransportError::Invalid("parse failed".to_string()))
        };

        let mut node = AeronSubscriberValueRefNode::new(subscriber, error_parser, 0);

        let result = node.core.poll_and_process();
        assert!(result.is_err());
    }

    #[test]
    fn given_parser_filter_when_cycle_then_option_wrapped_in_ok() {
        let msg = 42i64.to_le_bytes().to_vec();
        let subscriber = MockSubscriber::new(vec![msg]);

        let filter = |fragment: &[u8]| -> Option<i64> {
            if fragment.len() >= 8 {
                Some(i64::from_le_bytes(fragment[0..8].try_into().ok()?))
            } else {
                None
            }
        };

        // Wrap using the same pattern as parser_filter
        let wrapped = move |buf: &[u8]| -> Result<Option<i64>, TransportError> { Ok(filter(buf)) };

        let mut node = AeronSubscriberValueRefNode::new(subscriber, wrapped, 0);

        node.core.poll_and_process().unwrap();
        assert_eq!(*node.peek_ref(), 42);
    }

    // --- Dual-stream (aeron_sub) tests ---

    #[test]
    fn given_connected_subscriber_when_aeron_sub_then_status_transitions_to_connected() {
        let subscriber = ConnectedMockSubscriber::new(vec![], true);
        let (data_node, status) = aeron_sub(subscriber, fallible_i64_parser, 0i64);

        data_node.borrow_mut().core.poll_and_process().unwrap();

        assert_eq!(*status.borrow().get(), AeronStatus::Connected);
    }

    #[test]
    fn given_disconnected_subscriber_when_polled_then_status_stays_disconnected() {
        let subscriber = ConnectedMockSubscriber::new(vec![], false);
        let (data_node, status) = aeron_sub(subscriber, fallible_i64_parser, 0i64);

        data_node.borrow_mut().core.poll_and_process().unwrap();

        assert_eq!(*status.borrow().get(), AeronStatus::Disconnected);
    }

    #[test]
    fn given_status_unchanged_when_cycle_then_no_status_update() {
        let subscriber = ConnectedMockSubscriber::new(vec![], true);
        let (data_node, status) = aeron_sub(subscriber, fallible_i64_parser, 0i64);

        data_node.borrow_mut().core.poll_and_process().unwrap();
        assert_eq!(*status.borrow().get(), AeronStatus::Connected);

        data_node.borrow_mut().core.poll_and_process().unwrap();
        assert_eq!(*status.borrow().get(), AeronStatus::Connected);
    }

    #[test]
    fn given_valid_messages_when_aeron_sub_polled_then_data_node_updates() {
        let msg1 = 42i64.to_le_bytes().to_vec();
        let msg2 = 100i64.to_le_bytes().to_vec();
        let subscriber = ConnectedMockSubscriber::new(vec![msg1, msg2], false);
        let (data_node, _status) = aeron_sub(subscriber, fallible_i64_parser, 0i64);

        data_node.borrow_mut().core.poll_and_process().unwrap();

        assert_eq!(*data_node.borrow().peek_ref(), 100);
    }

    #[test]
    fn given_parser_error_when_aeron_sub_cycle_then_error_propagated() {
        let msg = 42i64.to_le_bytes().to_vec();
        let subscriber = ConnectedMockSubscriber::new(vec![msg], false);

        let error_parser = |_: &[u8]| -> Result<Option<i64>, TransportError> {
            Err(TransportError::Invalid("parse failed".to_string()))
        };

        let (data_node, _status) = aeron_sub(subscriber, error_parser, 0i64);

        let result = data_node.borrow_mut().core.poll_and_process();
        assert!(result.is_err());
    }

    #[test]
    fn given_connection_lost_when_cycle_then_status_transitions_to_disconnected() {
        let subscriber = ConnectedMockSubscriber::new(vec![], true);
        let (data_node, status) = aeron_sub(subscriber, fallible_i64_parser, 0i64);

        data_node.borrow_mut().core.poll_and_process().unwrap();
        assert_eq!(*status.borrow().get(), AeronStatus::Connected);

        data_node.borrow_mut().core.subscriber.connected = false;
        data_node.borrow_mut().core.poll_and_process().unwrap();
        assert_eq!(*status.borrow().get(), AeronStatus::Disconnected);
    }
}
