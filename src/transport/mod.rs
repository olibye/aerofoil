//! Aeron transport abstractions for message processing.
//!
//! This module provides trait-based abstractions for Aeron transport operations,
//! enabling zero-cost polymorphism across Rusteron and mock implementations for testing.

pub mod buffer;
pub mod error;

pub use buffer::{ClaimBuffer, FragmentBuffer, FragmentHeader};
pub use error::TransportError;

pub mod rusteron;

/// Publishes messages to an Aeron channel.
///
/// This trait provides three publication methods:
///
/// - [`offer`](AeronPublisher::offer): Accepts `&[u8]`
/// - [`offer_mut`](AeronPublisher::offer_mut): Accepts `&mut [u8]`
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

    /// Offers a message to the publication with a mutable buffer.
    ///
    /// # Returns
    ///
    /// - `Ok(position)` - The stream position where the message was published
    /// - `Err(TransportError::BackPressure)` - Buffer is full, retry later
    /// - `Err(_)` - Other transport error occurred
    fn offer_mut(&mut self, buffer: &mut [u8]) -> Result<i64, TransportError>;

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
}
