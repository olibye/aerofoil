//! Aeron-rs implementation of AeronSubscriber trait.

use crate::transport::{AeronSubscriber, FragmentBuffer, FragmentHeader, TransportError};
use aeron_rs::subscription::Subscription;
use std::cell::RefCell;

/// Aeron-rs based implementation of [`AeronSubscriber`].
///
/// This subscriber wraps an aeron-rs [`Subscription`] and implements the
/// `AeronSubscriber` trait using pure Rust.
///
/// # Deployment Benefits
///
/// - No C++ toolchain required
/// - Pure Rust memory safety
/// - Simpler cross-compilation
///
/// # Trade-offs vs Rusteron
///
/// - Less mature than Rusteron (C++ wrapper)
/// - May have different performance characteristics
/// - See `add-transport-benchmarks` for comparison data
pub struct AeronRsSubscriber {
    subscription: Subscription,
}

impl AeronRsSubscriber {
    /// Creates a new `AeronRsSubscriber` wrapping the given aeron-rs subscription.
    pub fn new(subscription: Subscription) -> Self {
        AeronRsSubscriber { subscription }
    }
}

impl AeronSubscriber for AeronRsSubscriber {
    fn poll<F>(&mut self, mut handler: F) -> Result<usize, TransportError>
    where
        F: FnMut(&FragmentBuffer) -> Result<(), TransportError>,
    {
        // Use RefCell to allow mutable access to handler in the closure
        let handler_cell = RefCell::new(&mut handler);
        let error_cell: RefCell<Option<TransportError>> = RefCell::new(None);

        // aeron-rs poll signature:
        // poll(&mut self, fragment_handler: &mut impl FnMut(&AtomicBuffer, Index, Index, &Header), fragment_limit: i32) -> i32
        let mut fragment_handler =
            |buffer: &aeron_rs::concurrent::atomic_buffer::AtomicBuffer,
             offset: i32,
             length: i32,
             header: &aeron_rs::concurrent::logbuffer::header::Header| {
                // Extract the data slice from the AtomicBuffer at the given offset
                // Safety: buffer is valid for the duration of this callback
                let data = unsafe {
                    std::slice::from_raw_parts(
                        (buffer.buffer() as *const u8).add(offset as usize),
                        length as usize,
                    )
                };

                // Create FragmentHeader from aeron-rs Header
                let frag_header = FragmentHeader {
                    position: header.position(),
                    session_id: header.session_id(),
                    stream_id: header.stream_id(),
                };

                // Create FragmentBuffer
                let fragment = FragmentBuffer::new(data, frag_header);

                // Call the user's handler
                if let Err(e) = handler_cell.borrow_mut()(&fragment) {
                    // Store the error and signal to stop polling
                    *error_cell.borrow_mut() = Some(e);
                }
            };

        // Poll the aeron-rs subscription with our adapter closure
        // fragment_limit: process 1 fragment per poll call (matches Rusteron behavior)
        let count = self.subscription.poll(&mut fragment_handler, 1);

        // Check if handler returned an error
        if let Some(err) = error_cell.into_inner() {
            return Err(err);
        }

        Ok(count as usize)
    }
}
