// ! Aeron transport abstractions for wingfoil message processing.
//!
//! This module provides trait-based abstractions for Aeron transport operations,
//! enabling zero-cost polymorphism across different Aeron client implementations
//! (Rusteron, aeron-rs) and mock implementations for testing.
//!
//! # Design Philosophy
//!
//! - **Zero-cost abstraction**: Traits use static dispatch (monomorphization),
//!   not dynamic dispatch, ensuring no runtime overhead.
//! - **Non-blocking**: All transport operations are guaranteed non-blocking,
//!   critical for low-latency high-frequency trading systems.
//! - **Zero-copy**: Buffer types use lifetime bounds to enable zero-copy
//!   message handling where the underlying client supports it.
//! - **Testable**: Mock implementations allow testing without Aeron infrastructure.
//!
//! # Key Design Decisions
//!
//! ## Decision 1: Separate Publisher and Subscriber Traits
//!
//! We define `AeronPublisher` and `AeronSubscriber` as separate traits rather than
//! a single `Transport` trait. This provides:
//!
//! - **Focused APIs**: Most code uses either publishing or subscribing, not both
//! - **Smaller surface area**: Easier to implement and mock
//! - **Type-level enforcement**: A processor that only publishes can't accidentally subscribe
//! - **Single responsibility**: Each trait has one clear purpose
//!
//! ## Decision 2: Unified Error Type
//!
//! All trait methods return `Result<T, TransportError>` for:
//!
//! - **Consistent error handling**: Same error type across all implementations
//! - **Ergonomic code**: Can use `?` operator throughout
//! - **Extensibility**: Error enum can be extended without breaking existing code
//! - **Backend details preserved**: Via `source()` method for debugging
//!
//! ## Decision 3: Lifetime-Bound Buffer Types
//!
//! `ClaimBuffer<'a>` and `FragmentBuffer<'a>` borrow from the underlying transport:
//!
//! - **Safety**: Prevents use-after-free when accessing Aeron-managed buffers
//! - **Zero-copy semantics**: Buffers are views, not owned data
//! - **Compiler-enforced**: Rust lifetime checker ensures correctness
//! - **Performance**: More efficient than copying or reference counting
//!
//! ## Decision 4: Manual Test Implementations
//!
//! Instead of using mockall's `#[automock]`, we design traits to be simple enough
//! for users to implement manually in tests:
//!
//! - **Mockall limitations**: Cannot handle complex lifetimes (`try_claim<'a>`)
//!   or generic closures (`poll<F>`)
//! - **Simplicity**: Manual implementations are ~10 lines of straightforward code
//! - **No dependencies**: Avoids mockall dependency and its limitations
//! - **Flexibility**: Users implement exactly what they need for their tests
//!
//! ## Decision 5: Non-Blocking Contract
//!
//! All trait methods are documented to never block:
//!
//! - **HFT requirement**: Blocking is unacceptable in high-frequency trading
//! - **Documented contract**: Clear expectation in trait rustdoc
//! - **Back-pressure handling**: Explicit `BackPressure` error when buffers are full
//! - **Safe for latency-sensitive code**: Callers can rely on immediate return
//!
//! # Architecture
//!
//! The transport layer consists of two main traits:
//!
//! - [`AeronPublisher`]: Publishes messages to Aeron channels
//! - [`AeronSubscriber`]: Receives messages from Aeron channels
//!
//! These traits can be implemented by:
//!
//! - Rusteron adapter (wraps C++ Aeron client) - feature `rusteron`
//! - aeron-rs adapter (pure Rust client) - feature `aeron-rs`
//! - Mock implementations for testing (always available)
//!
//! # Example
//!
//! ```ignore
//! use aerofoil::transport::{AeronPublisher, TransportError};
//!
//! fn send_message<P: AeronPublisher>(
//!     publisher: &mut P,
//!     message: &[u8]
//! ) -> Result<i64, TransportError> {
//!     publisher.offer(message)
//! }
//!
//! // Works with any implementation: Rusteron, aeron-rs, or mocks
//! ```

pub mod buffer;
pub mod error;

pub use buffer::{ClaimBuffer, FragmentBuffer, FragmentHeader};
pub use error::TransportError;

#[cfg(feature = "rusteron")]
pub mod rusteron;

#[cfg(test)]
mod tests;

/// Publishes messages to an Aeron channel.
///
/// This trait provides two publication methods:
///
/// - [`offer`](AeronPublisher::offer): Copy message data into Aeron buffer (simpler)
/// - [`try_claim`](AeronPublisher::try_claim): Claim buffer for zero-copy writing (faster)
///
/// All methods are **guaranteed non-blocking** and will return immediately,
/// even under back-pressure conditions.
///
/// # Implementation Requirements
///
/// Implementations MUST:
///
/// - Never block or sleep
/// - Return [`TransportError::BackPressure`] immediately when buffers are full
/// - Be thread-safe if the underlying Aeron client supports it
///
/// # Example
///
/// ```ignore
/// # use aerofoil::transport::{AeronPublisher, TransportError};
/// # fn example<P: AeronPublisher>(publisher: &mut P) -> Result<(), TransportError> {
/// // Simple publish (copies data)
/// let position = publisher.offer(b"hello world")?;
/// println!("Published at position {}", position);
///
/// // Zero-copy publish
/// let mut claim = publisher.try_claim(256)?;
/// claim[0..5].copy_from_slice(b"hello");
/// drop(claim); // commit happens on drop
/// # Ok(())
/// # }
/// ```
pub trait AeronPublisher {
    /// Offers a message to the publication.
    ///
    /// This method copies the provided buffer into the Aeron publication buffer.
    /// It is non-blocking and will return immediately with back-pressure indication
    /// if the buffer is full.
    ///
    /// # Arguments
    ///
    /// * `buffer` - The message data to publish
    ///
    /// # Returns
    ///
    /// - `Ok(position)` - The stream position where the message was published
    /// - `Err(TransportError::BackPressure)` - Buffer is full, retry later
    /// - `Err(_)` - Other transport error occurred
    ///
    /// # Non-blocking Guarantee
    ///
    /// This method will never block. If the Aeron buffer is full, it returns
    /// `TransportError::BackPressure` immediately.
    fn offer(&mut self, buffer: &[u8]) -> Result<i64, TransportError>;

    /// Claims a buffer for zero-copy message writing.
    ///
    /// This method provides direct access to the Aeron publication buffer,
    /// allowing messages to be written without an intermediate copy. This is
    /// the lowest-latency publication path.
    ///
    /// # Arguments
    ///
    /// * `length` - The number of bytes to claim
    ///
    /// # Returns
    ///
    /// - `Ok(ClaimBuffer)` - A mutable buffer that can be written to
    /// - `Err(TransportError::BackPressure)` - Buffer is full, retry later
    /// - `Err(_)` - Other transport error occurred
    ///
    /// # Non-blocking Guarantee
    ///
    /// This method will never block. If the Aeron buffer is full, it returns
    /// `TransportError::BackPressure` immediately.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut claim = publisher.try_claim(256)?;
    /// // Write directly into the buffer
    /// claim[0..5].copy_from_slice(b"hello");
    /// // Buffer is committed on drop
    /// ```
    fn try_claim<'a>(&'a mut self, length: usize) -> Result<ClaimBuffer<'a>, TransportError>;
}

/// Subscribes to and receives messages from an Aeron channel.
///
/// This trait provides a polling interface for receiving messages without blocking.
/// Messages are delivered via a callback handler, following Aeron's fragment handler
/// pattern.
///
/// # Implementation Requirements
///
/// Implementations MUST:
///
/// - Never block or sleep
/// - Return immediately even when no messages are available (with count of 0)
/// - Invoke the handler for each received fragment
/// - Provide zero-copy access to message data via [`FragmentBuffer`]
///
/// # Example
///
/// ```ignore
/// # use aerofoil::transport::{AeronSubscriber, TransportError};
/// # fn example<S: AeronSubscriber>(subscriber: &mut S) -> Result<(), TransportError> {
/// let count = subscriber.poll(|fragment| {
///     println!("Received {} bytes at position {}",
///              fragment.len(),
///              fragment.position());
///     // Process fragment data
///     Ok(())
/// })?;
/// println!("Processed {} fragments", count);
/// # Ok(())
/// # }
/// ```
pub trait AeronSubscriber {
    /// Polls for messages and delivers them to the provided handler.
    ///
    /// This method checks for available messages and invokes the handler for each
    /// received fragment. It is non-blocking and will return immediately, even if
    /// no messages are available.
    ///
    /// # Arguments
    ///
    /// * `handler` - A callback invoked for each received message fragment
    ///
    /// # Returns
    ///
    /// - `Ok(count)` - The number of fragments processed
    /// - `Err(_)` - A transport error occurred, or the handler returned an error
    ///
    /// # Non-blocking Guarantee
    ///
    /// This method will never block. If no messages are available, it returns
    /// immediately with a count of 0.
    ///
    /// # Handler Errors
    ///
    /// If the handler returns an error, polling stops and the error is propagated.
    /// This allows error-based flow control.
    ///
    /// # Example
    ///
    /// ```ignore
    /// subscriber.poll(|fragment| {
    ///     // Zero-copy access to message data
    ///     let data: &[u8] = fragment.as_ref();
    ///     process_message(data)?;
    ///     Ok(())
    /// })?;
    /// ```
    fn poll<F>(&mut self, handler: F) -> Result<usize, TransportError>
    where
        F: FnMut(&FragmentBuffer) -> Result<(), TransportError>;
}
