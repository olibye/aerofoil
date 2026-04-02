//! Builder pattern for Aeron subscriber nodes.
//!
//! This module provides [`AeronSubscriberNodeBuilder`] which handles
//! `Rc<RefCell<>>` wrapping for Wingfoil graph integration.
//!
//! # Example
//!
//! See `examples/subscriber_node_value_access.rs` for a complete example.

use crate::transport::{AeronSubscriber, TransportError};
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
    F: FnMut(&[u8]) -> Result<Option<T>, TransportError>,
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
    F: FnMut(&[u8]) -> Result<Option<T>, TransportError> + 'static,
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

    /// Sets a fallible parser that returns `Result<Option<T>, TransportError>`.
    ///
    /// Use this when the parser needs to report errors back to the graph runner.
    /// For infallible parsers that return `Option<T>`, use [`.parser()`](AeronSubscriberNodeBuilder::parser) instead.
    ///
    /// - `Ok(Some(value))` — successfully parsed
    /// - `Ok(None)` — message skipped (e.g. wrong type)
    /// - `Err(e)` — parse error propagated to the graph
    pub fn try_parser(mut self, parser: F) -> Self {
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
    F: FnMut(&[u8]) -> Result<Option<T>, TransportError> + 'static,
    S: AeronSubscriber + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Infallible parser support.
///
/// The `.parser()` method wraps an `FnMut(&[u8]) -> Option<T>` into
/// `Ok(f(buf))`, providing the common-case API for parsers that don't
/// need error propagation.
impl<T, S> AeronSubscriberNodeBuilder<T, fn(&[u8]) -> Result<Option<T>, TransportError>, S>
where
    T: Element + 'static,
    S: AeronSubscriber + 'static,
{
    /// Sets an infallible parser that returns `Option<T>`, wrapping it into `Ok(f(buf))`.
    ///
    /// This is the common-case API. For parsers that need to report errors,
    /// use [`.try_parser()`](AeronSubscriberNodeBuilder::try_parser) instead.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let node = AeronSubscriberValueNode::builder()
    ///     .subscriber(subscriber)
    ///     .parser(|buf| {
    ///         if buf.len() >= 8 {
    ///             Some(i64::from_le_bytes(buf[0..8].try_into().ok()?))
    ///         } else {
    ///             None
    ///         }
    ///     })
    ///     .default(0i64)
    ///     .build();
    /// ```
    #[allow(clippy::type_complexity)]
    pub fn parser<G: FnMut(&[u8]) -> Option<T> + 'static>(
        self,
        mut parser: G,
    ) -> AeronSubscriberNodeBuilder<
        T,
        impl FnMut(&[u8]) -> Result<Option<T>, TransportError> + 'static,
        S,
    > {
        AeronSubscriberNodeBuilder {
            subscriber: self.subscriber,
            parser: Some(move |buf: &[u8]| -> Result<Option<T>, TransportError> {
                Ok(parser(buf))
            }),
            default_value: self.default_value,
            _marker: PhantomData,
        }
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

    fn i64_try_parser(fragment: &[u8]) -> Result<Option<i64>, TransportError> {
        if fragment.len() >= 8 {
            let bytes: [u8; 8] = fragment[0..8].try_into().unwrap();
            Ok(Some(i64::from_le_bytes(bytes)))
        } else {
            Ok(None)
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
    fn given_builder_when_try_parser_set_then_builds_valid_node() {
        let subscriber = MockSubscriber::new(vec![42i64.to_le_bytes().to_vec()]);

        let node = AeronSubscriberNodeBuilder::new()
            .subscriber(subscriber)
            .try_parser(i64_try_parser)
            .default(0i64)
            .build();

        assert!(Rc::strong_count(&node) >= 1);
    }

    #[test]
    fn given_builder_when_build_ref_called_then_builds_value_ref_node() {
        let subscriber = MockSubscriber::new(vec![]);

        let node = AeronSubscriberNodeBuilder::new()
            .subscriber(subscriber)
            .try_parser(i64_try_parser)
            .default(0i64)
            .build_ref();

        assert_eq!(*node.borrow().peek_ref(), 0);
    }

    #[test]
    fn given_node_when_peek_value_called_then_returns_value() {
        let subscriber = MockSubscriber::new(vec![99i64.to_le_bytes().to_vec()]);
        let node = AeronSubscriberNodeBuilder::new()
            .subscriber(subscriber)
            .try_parser(i64_try_parser)
            .default(0i64)
            .build();

        let value = node.peek_value();

        assert_eq!(value, 0);
    }

    #[test]
    #[should_panic(expected = "subscriber is required")]
    fn given_builder_when_subscriber_missing_then_panics() {
        let _ = AeronSubscriberNodeBuilder::<i64, _, MockSubscriber>::new()
            .try_parser(i64_try_parser)
            .default(0i64)
            .build();
    }

    #[test]
    #[should_panic(expected = "parser is required")]
    fn given_builder_when_parser_missing_then_panics() {
        let subscriber = MockSubscriber::new(vec![]);
        let builder = AeronSubscriberNodeBuilder::<
            i64,
            fn(&[u8]) -> Result<Option<i64>, TransportError>,
            _,
        >::new()
        .subscriber(subscriber)
        .default(0i64);

        let _ = builder.build();
    }

    #[test]
    #[should_panic(expected = "default value is required")]
    fn given_builder_when_default_missing_then_panics() {
        let subscriber = MockSubscriber::new(vec![]);

        let _ = AeronSubscriberNodeBuilder::new()
            .subscriber(subscriber)
            .try_parser(i64_try_parser)
            .build();
    }

    #[test]
    fn given_parser_when_build_then_wraps_option_in_ok() {
        let subscriber = MockSubscriber::new(vec![42i64.to_le_bytes().to_vec()]);

        let node = AeronSubscriberValueNode::builder()
            .subscriber(subscriber)
            .parser(i64_parser)
            .default(0i64)
            .build();

        assert_eq!(node.peek_value(), 0);
    }

    #[test]
    fn given_parser_when_build_ref_then_wraps_option_in_ok() {
        let subscriber = MockSubscriber::new(vec![]);

        let node = AeronSubscriberValueRefNode::builder()
            .subscriber(subscriber)
            .parser(i64_parser)
            .default(0i64)
            .build_ref();

        assert_eq!(*node.borrow().peek_ref(), 0);
    }
}
