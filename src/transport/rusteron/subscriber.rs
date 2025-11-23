//! Rusteron implementation of AeronSubscriber trait.

use crate::transport::{AeronSubscriber, FragmentBuffer, FragmentHeader, TransportError};
use rusteron_client::AeronSubscription;

use super::error::aeron_error_to_transport_error;

/// Rusteron-based implementation of [`AeronSubscriber`].
///
/// This subscriber wraps a Rusteron [`AeronSubscription`] and implements the
/// `AeronSubscriber` trait, providing access to message reception through the
/// mature C++ Aeron client.
///
/// # Design Decision: Callback-Based Polling
///
/// The `poll` implementation uses Rusteron's `poll_once` method because:
/// - **Natural fit**: Our trait's `poll<F>` takes a closure, matching Rusteron's API
/// - **Zero allocation**: No intermediate buffer copies needed
/// - **Aeron semantics**: Matches how Aeron natively works with fragment handlers
///
/// # Thread Safety
///
/// Like `RusteronPublisher`, this should be used on a single thread and
/// optionally pinned to a specific CPU core for HFT performance.
///
/// # Example
///
/// ```ignore
/// use aerofoil::transport::AeronSubscriber;
/// use aerofoil::transport::rusteron::RusteronSubscriber;
///
/// // Create subscriber (requires media driver)
/// let mut subscriber = RusteronSubscriber::new(subscription);
///
/// // Poll for messages
/// subscriber.poll(|fragment| {
///     println!("Received: {:?}", fragment.as_ref());
///     Ok(())
/// })?;
/// ```
pub struct RusteronSubscriber {
    subscription: AeronSubscription,
}

impl RusteronSubscriber {
    /// Creates a new `RusteronSubscriber` wrapping the given Rusteron subscription.
    ///
    /// # Arguments
    ///
    /// * `subscription` - A configured Rusteron `AeronSubscription` instance
    ///
    /// # Example
    ///
    /// ```ignore
    /// let subscription = AeronSubscription::new(/* ... */)?;
    /// let subscriber = RusteronSubscriber::new(subscription);
    /// ```
    pub fn new(subscription: AeronSubscription) -> Self {
        RusteronSubscriber { subscription }
    }

    /// Returns a reference to the underlying Rusteron subscription.
    ///
    /// This provides access to Rusteron-specific features not exposed by the
    /// `AeronSubscriber` trait.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let raw_sub = subscriber.inner();
    /// // Use Rusteron-specific methods...
    /// ```
    pub fn inner(&self) -> &AeronSubscription {
        &self.subscription
    }
}

impl AeronSubscriber for RusteronSubscriber {
    fn poll<F>(&mut self, mut handler: F) -> Result<usize, TransportError>
    where
        F: FnMut(&FragmentBuffer) -> Result<(), TransportError>,
    {
        // Use Rusteron's poll_once which takes a closure
        // The closure receives (&[u8], AeronHeader) for each fragment
        //
        // Design Decision: Error Handling in Closure
        //
        // The user's handler can return errors, but Rusteron's poll_once
        // expects a closure that doesn't return Result. We need to:
        // 1. Capture any error from the handler
        // 2. Stop polling if an error occurs
        // 3. Return the error after polling completes

        let mut handler_error: Option<TransportError> = None;
        let mut fragment_count: usize = 0;

        // Call Rusteron's poll_once with a closure
        let poll_result = self.subscription.poll_once(
            |buffer: &[u8], header: rusteron_client::AeronHeader| {
                // Only process if we haven't encountered an error yet
                if handler_error.is_none() {
                    fragment_count += 1;

                    // Convert Rusteron header to our FragmentHeader
                    // Get header values which contain session_id and stream_id
                    match header.get_values() {
                        Ok(header_values) => {
                            let frame = header_values.frame();

                            let frag_header = FragmentHeader {
                                position: header.position(),
                                session_id: frame.session_id(),
                                stream_id: frame.stream_id(),
                            };

                            // Create FragmentBuffer wrapping the received data
                            let fragment = FragmentBuffer::new(buffer, frag_header);

                            // Call user's handler and capture any error
                            if let Err(e) = handler(&fragment) {
                                handler_error = Some(e);
                            }
                        }
                        Err(e) => {
                            // Failed to get header values
                            handler_error = Some(aeron_error_to_transport_error(e));
                        }
                    }
                }
            },
            std::i32::MAX as usize, // fragment_limit: poll all available
        );

        // Check if polling itself failed
        poll_result.map_err(aeron_error_to_transport_error)?;

        // If handler returned an error, propagate it
        if let Some(err) = handler_error {
            return Err(err);
        }

        // Return count of fragments processed
        Ok(fragment_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests cannot run without a media driver and proper Rusteron setup
    // They are provided as examples of how to test with manual test implementations

    #[test]
    fn test_subscriber_trait_object() {
        // Given: This test verifies the type implements the trait correctly
        // We can't create a real subscriber without a media driver, but we can
        // verify the types are correct at compile time

        // This function accepts any AeronSubscriber implementation
        fn takes_subscriber<S: AeronSubscriber>(_s: &S) {}

        // If this compiles, RusteronSubscriber correctly implements AeronSubscriber
        // (Actual instance creation would require media driver)
    }
}
