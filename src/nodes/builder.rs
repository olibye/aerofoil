//! Builder pattern for Aeron subscriber nodes.
//!
//! This module provides [`AeronSubscriberNodeBuilder`] which handles
//! `Rc<RefCell<>>` wrapping for Wingfoil graph integration.
//!
//! # Example
//!
//! See `examples/subscriber_node_value_access.rs` for a complete example.

use crate::transport::AeronSubscriber;
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;
use wingfoil::Element;

use super::{AeronSubscriberValueNode, AeronSubscriberValueRefNode};

/// Builder for constructing Aeron subscriber nodes with automatic `Rc<RefCell<>>` wrapping.
///
/// Returns `Rc<RefCell<Node>>` which can be cloned for the graph (coerces to `Rc<dyn Node>`)
/// and used directly as an upstream reference for downstream nodes.
///
/// # Type Parameters
///
/// - `T`: The message type (must implement `Element`)
/// - `F`: The parser function type
/// - `S`: The subscriber implementation type
pub struct AeronSubscriberNodeBuilder<T, F, S>
where
    T: Element,
    F: FnMut(&[u8]) -> Option<T>,
    S: AeronSubscriber,
{
    subscriber: Option<S>,
    parser: Option<F>,
    default_value: Option<T>,
    _marker: PhantomData<T>,
}

impl<T, F, S> AeronSubscriberNodeBuilder<T, F, S>
where
    T: Element,
    F: FnMut(&[u8]) -> Option<T> + 'static,
    S: AeronSubscriber + 'static,
{
    /// Creates a new builder with no fields set.
    pub fn new() -> Self {
        Self {
            subscriber: None,
            parser: None,
            default_value: None,
            _marker: PhantomData,
        }
    }

    /// Sets the Aeron subscriber to poll for messages.
    pub fn subscriber(mut self, subscriber: S) -> Self {
        self.subscriber = Some(subscriber);
        self
    }

    /// Sets the parser function for converting byte fragments to typed values.
    pub fn parser(mut self, parser: F) -> Self {
        self.parser = Some(parser);
        self
    }

    /// Sets the default value before any messages are received.
    pub fn default(mut self, value: T) -> Self {
        self.default_value = Some(value);
        self
    }

    /// Builds an `AeronSubscriberValueNode` for value-based access.
    ///
    /// Returns `Rc<RefCell<AeronSubscriberValueNode<T, F, S>>>` which can be:
    /// - Cloned and added to a Wingfoil graph (coerces to `Rc<dyn Node>`)
    /// - Used directly by downstream nodes via `peek_value()`
    ///
    /// # Panics
    ///
    /// Panics if subscriber, parser, or default value is not set.
    pub fn build(self) -> Rc<RefCell<AeronSubscriberValueNode<T, F, S>>> {
        let subscriber = self
            .subscriber
            .expect("AeronSubscriberNodeBuilder: subscriber is required");
        let parser = self
            .parser
            .expect("AeronSubscriberNodeBuilder: parser is required");
        let default_value = self
            .default_value
            .expect("AeronSubscriberNodeBuilder: default value is required");

        Rc::new(RefCell::new(AeronSubscriberValueNode::new(
            subscriber,
            parser,
            default_value,
        )))
    }

    /// Builds an `AeronSubscriberValueRefNode` for reference-based access.
    ///
    /// Returns `Rc<RefCell<AeronSubscriberValueRefNode<T, F, S>>>` which can be:
    /// - Cloned and added to a Wingfoil graph (coerces to `Rc<dyn Node>`)
    /// - Used directly by downstream nodes via `peek_ref()`
    ///
    /// # Panics
    ///
    /// Panics if subscriber, parser, or default value is not set.
    pub fn build_ref(self) -> Rc<RefCell<AeronSubscriberValueRefNode<T, F, S>>> {
        let subscriber = self
            .subscriber
            .expect("AeronSubscriberNodeBuilder: subscriber is required");
        let parser = self
            .parser
            .expect("AeronSubscriberNodeBuilder: parser is required");
        let default_value = self
            .default_value
            .expect("AeronSubscriberNodeBuilder: default value is required");

        Rc::new(RefCell::new(AeronSubscriberValueRefNode::new(
            subscriber,
            parser,
            default_value,
        )))
    }
}

impl<T, F, S> Default for AeronSubscriberNodeBuilder<T, F, S>
where
    T: Element,
    F: FnMut(&[u8]) -> Option<T> + 'static,
    S: AeronSubscriber + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::{FragmentBuffer, FragmentHeader, TransportError};
    use wingfoil::{StreamPeek, StreamPeekRef};

    /// Mock subscriber for testing
    struct MockSubscriber {
        messages: RefCell<Vec<Vec<u8>>>,
    }

    impl MockSubscriber {
        fn new(messages: Vec<Vec<u8>>) -> Self {
            Self {
                messages: RefCell::new(messages),
            }
        }
    }

    impl AeronSubscriber for MockSubscriber {
        fn poll<H>(&mut self, mut handler: H) -> Result<usize, TransportError>
        where
            H: FnMut(&FragmentBuffer) -> Result<(), TransportError>,
        {
            let mut messages = self.messages.borrow_mut();
            let count = messages.len();

            for message in messages.drain(..) {
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

    fn i64_parser(fragment: &[u8]) -> Option<i64> {
        if fragment.len() >= 8 {
            let bytes: [u8; 8] = fragment[0..8].try_into().ok()?;
            Some(i64::from_le_bytes(bytes))
        } else {
            None
        }
    }

    #[test]
    fn given_builder_when_all_fields_set_then_builds_valid_node() {
        // Given: A builder with all fields set
        let subscriber = MockSubscriber::new(vec![42i64.to_le_bytes().to_vec()]);

        // When: build() is called
        let node = AeronSubscriberNodeBuilder::new()
            .subscriber(subscriber)
            .parser(i64_parser)
            .default(0i64)
            .build();

        // Then: Returns valid Rc<RefCell<Node>>
        assert!(Rc::strong_count(&node) >= 1);
    }

    #[test]
    fn given_builder_when_build_ref_called_then_builds_value_ref_node() {
        // Given: A builder with all fields set
        let subscriber = MockSubscriber::new(vec![]);

        // When: build_ref() is called
        let node = AeronSubscriberNodeBuilder::new()
            .subscriber(subscriber)
            .parser(i64_parser)
            .default(0i64)
            .build_ref();

        // Then: node can access values via peek_ref
        assert_eq!(*node.borrow().peek_ref(), 0);
    }

    #[test]
    fn given_node_when_peek_value_called_then_returns_value() {
        // Given: A built node with messages
        let subscriber = MockSubscriber::new(vec![99i64.to_le_bytes().to_vec()]);
        let node = AeronSubscriberNodeBuilder::new()
            .subscriber(subscriber)
            .parser(i64_parser)
            .default(0i64)
            .build();

        // When: peek_value is called (before polling, returns default)
        let value = node.peek_value();

        // Then: Returns the default value
        assert_eq!(value, 0);
    }

    #[test]
    #[should_panic(expected = "subscriber is required")]
    fn given_builder_when_subscriber_missing_then_panics() {
        // Given: A builder without subscriber
        // When: build() is called
        // Then: Panics with clear message
        let _ = AeronSubscriberNodeBuilder::<i64, _, MockSubscriber>::new()
            .parser(i64_parser)
            .default(0i64)
            .build();
    }

    #[test]
    #[should_panic(expected = "parser is required")]
    fn given_builder_when_parser_missing_then_panics() {
        // Given: A builder without parser
        let subscriber = MockSubscriber::new(vec![]);
        let builder = AeronSubscriberNodeBuilder::<i64, fn(&[u8]) -> Option<i64>, _>::new()
            .subscriber(subscriber)
            .default(0i64);

        // When: build() is called
        // Then: Panics with clear message
        let _ = builder.build();
    }

    #[test]
    #[should_panic(expected = "default value is required")]
    fn given_builder_when_default_missing_then_panics() {
        // Given: A builder without default value
        let subscriber = MockSubscriber::new(vec![]);

        // When: build() is called
        // Then: Panics with clear message
        let _ = AeronSubscriberNodeBuilder::new()
            .subscriber(subscriber)
            .parser(i64_parser)
            .build();
    }
}
