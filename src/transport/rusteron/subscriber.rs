//! Rusteron implementation of AeronSubscriber trait.

use crate::transport::{AeronSubscriber, FragmentBuffer, FragmentHeader, TransportError};
use rusteron_client::AeronSubscription;
use std::cell::RefCell;

/// Rusteron-based implementation of [`AeronSubscriber`].
///
/// This subscriber wraps a Rusteron [`AeronSubscription`] and implements the
/// `AeronSubscriber` trait.
pub struct RusteronSubscriber {
    subscription: AeronSubscription,
}

impl std::fmt::Debug for RusteronSubscriber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RusteronSubscriber").finish_non_exhaustive()
    }
}

impl RusteronSubscriber {
    /// Creates a new `RusteronSubscriber` wrapping the given Rusteron subscription.
    pub fn new(subscription: AeronSubscription) -> Self {
        RusteronSubscriber { subscription }
    }
}

impl AeronSubscriber for RusteronSubscriber {
    fn is_connected(&self) -> bool {
        self.subscription.is_connected()
    }

    fn is_closed(&self) -> bool {
        self.subscription.is_closed()
    }

    fn poll<F>(&mut self, mut handler: F) -> Result<usize, TransportError>
    where
        F: FnMut(&FragmentBuffer) -> Result<(), TransportError>,
    {
        // Use RefCell to allow mutable access to handler in the closure
        let handler_cell = RefCell::new(&mut handler);
        let error_cell: RefCell<Option<TransportError>> = RefCell::new(None);

        // Poll the Rusteron subscription with our adapter closure
        let count_result = self.subscription.poll_once(
            |buffer: &[u8], header: rusteron_client::AeronHeader| {
                // Create FragmentHeader from Rusteron header
                // Note: Rusteron's AeronHeader is passed by value, and we extract
                // the limited metadata available
                let frag_header = FragmentHeader {
                    position: header.position(),
                    session_id: 0, // Rusteron AeronHeader doesn't expose session_id directly
                    stream_id: 0,  // Rusteron AeronHeader doesn't expose stream_id directly
                };

                // Create FragmentBuffer
                let fragment = FragmentBuffer::new(buffer, frag_header);

                // Call the user's handler
                if let Err(e) = handler_cell.borrow_mut()(&fragment) {
                    // Store the error and signal to stop polling
                    *error_cell.borrow_mut() = Some(e);
                }
            },
            1, // fragment_limit: process 1 fragment per poll call
        );

        // Check if handler returned an error
        if let Some(err) = error_cell.into_inner() {
            return Err(err);
        }

        // Rusteron returns Result<i32, AeronCError>
        match count_result {
            Ok(count) => Ok(count as usize),
            Err(e) => Err(TransportError::Backend(format!(
                "Rusteron poll error: {:?}",
                e
            ))),
        }
    }
}
