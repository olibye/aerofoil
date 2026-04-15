//! Aeron transport abstractions for message processing.
//!
//! This module provides trait-based abstractions for Aeron transport operations,
//! enabling zero-cost polymorphism across Rusteron and mock implementations for testing.

pub mod buffer;
pub mod channel;
pub mod discovery;
pub mod error;

pub use buffer::{ClaimBuffer, FragmentBuffer, FragmentHeader};
pub use channel::ChannelUri;
pub use discovery::{
    aeron_pub_named, aeron_sub_discover, lookup_pub, lookup_sub, register_pub, register_sub,
    DiscoveryError,
};
pub use error::TransportError;

pub mod rusteron;

/// Lifecycle status of an Aeron transport endpoint.
///
/// Represents the connection state of a publisher or subscriber,
/// enabling status monitoring and lifecycle tracking.
#[derive(Debug, Clone, PartialEq, Default)]
#[non_exhaustive]
pub enum AeronStatus {
    /// The endpoint is connected and actively communicating.
    Connected,
    /// The endpoint is not connected (initial state).
    #[default]
    Disconnected,
    /// The endpoint is experiencing back-pressure.
    BackPressured,
    /// The endpoint has been closed.
    Closed,
}

/// Publishes messages to an Aeron channel.
///
/// This trait provides two publication methods:
///
/// - [`offer`](AeronPublisher::offer): Accepts `&[u8]`
/// - [`try_claim`](AeronPublisher::try_claim): Claim buffer for direct writing
///
/// All methods are **guaranteed non-blocking** and will return immediately,
/// even under back-pressure conditions.
pub trait AeronPublisher {
    /// Offers a message to the publication.
    ///
    /// # Returns
    ///
    /// - `Ok(position)` - The stream position where the message was published
    /// - `Err(TransportError::BackPressure)` - Buffer is full, retry later
    /// - `Err(_)` - Other transport error occurred
    fn offer(&mut self, buffer: &[u8]) -> Result<i64, TransportError>;

    /// Claims a buffer for zero-copy message writing.
    ///
    /// This method provides direct access to the Aeron publication buffer,
    /// allowing messages to be written without an intermediate copy.
    ///
    /// # Returns
    ///
    /// - `Ok(ClaimBuffer)` - A mutable buffer that can be written to
    /// - `Err(TransportError::BackPressure)` - Buffer is full, retry later
    /// - `Err(_)` - Other transport error occurred
    fn try_claim<'a>(&'a mut self, length: usize) -> Result<ClaimBuffer<'a>, TransportError>;

    /// Returns whether this publication is currently connected to at least one subscriber.
    ///
    /// # Returns
    ///
    /// `true` if connected, `false` otherwise. Default implementation returns `false`.
    fn is_connected(&self) -> bool {
        false
    }

    /// Returns whether this publication has been closed.
    ///
    /// A closed publication has had its lifecycle ended (gracefully or via shutdown).
    /// This is distinct from being temporarily disconnected — closed is terminal.
    ///
    /// # Returns
    ///
    /// `true` if closed, `false` otherwise. Default implementation returns `false`.
    fn is_closed(&self) -> bool {
        false
    }
}

/// Subscribes to and receives messages from an Aeron channel.
///
/// This trait provides a polling interface for receiving messages without blocking.
/// Messages are delivered via a callback handler, following Aeron's fragment handler
/// pattern.
pub trait AeronSubscriber {
    /// Polls for messages and delivers them to the provided handler.
    ///
    /// This method checks for available messages and invokes the handler for each
    /// received fragment. It is non-blocking and will return immediately, even if
    /// no messages are available.
    ///
    /// # Returns
    ///
    /// - `Ok(count)` - The number of fragments processed
    /// - `Err(_)` - A transport error occurred, or the handler returned an error
    fn poll<F>(&mut self, handler: F) -> Result<usize, TransportError>
    where
        F: FnMut(&FragmentBuffer) -> Result<(), TransportError>;

    /// Returns whether this subscription is currently connected to at least one publication.
    ///
    /// # Returns
    ///
    /// `true` if connected, `false` otherwise. Default implementation returns `false`.
    fn is_connected(&self) -> bool {
        false
    }

    /// Returns whether this subscription has been closed.
    ///
    /// A closed subscription has had its lifecycle ended (gracefully or via shutdown).
    /// This is distinct from being temporarily disconnected — closed is terminal.
    ///
    /// # Returns
    ///
    /// `true` if closed, `false` otherwise. Default implementation returns `false`.
    fn is_closed(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockPublisher;

    impl AeronPublisher for MockPublisher {
        fn offer(&mut self, _buffer: &[u8]) -> Result<i64, TransportError> {
            Ok(0)
        }

        fn try_claim<'a>(&'a mut self, _length: usize) -> Result<ClaimBuffer<'a>, TransportError> {
            Err(TransportError::Invalid("mock".to_string()))
        }
    }

    struct MockSubscriber;

    impl AeronSubscriber for MockSubscriber {
        fn poll<F>(&mut self, _handler: F) -> Result<usize, TransportError>
        where
            F: FnMut(&FragmentBuffer) -> Result<(), TransportError>,
        {
            Ok(0)
        }
    }

    struct ConnectedSubscriber;

    impl AeronSubscriber for ConnectedSubscriber {
        fn poll<F>(&mut self, _handler: F) -> Result<usize, TransportError>
        where
            F: FnMut(&FragmentBuffer) -> Result<(), TransportError>,
        {
            Ok(0)
        }

        fn is_connected(&self) -> bool {
            true
        }
    }

    #[test]
    fn given_aeron_status_when_default_then_disconnected() {
        let status = AeronStatus::default();
        assert_eq!(status, AeronStatus::Disconnected);
    }

    #[test]
    fn given_aeron_status_when_clone_then_equal() {
        let status = AeronStatus::Connected;
        let cloned = status.clone();
        assert_eq!(status, cloned);
    }

    #[test]
    fn given_publisher_when_is_connected_default_then_returns_false() {
        let publisher = MockPublisher;
        assert!(!publisher.is_connected());
    }

    #[test]
    fn given_subscriber_when_is_connected_default_then_returns_false() {
        let subscriber = MockSubscriber;
        assert!(!subscriber.is_connected());
    }

    #[test]
    fn given_subscriber_when_is_connected_overridden_then_returns_true() {
        let subscriber = ConnectedSubscriber;
        assert!(subscriber.is_connected());
    }
}
