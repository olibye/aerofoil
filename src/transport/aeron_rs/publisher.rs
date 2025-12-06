//! Aeron-rs implementation of AeronPublisher trait.

use crate::transport::{AeronPublisher, ClaimBuffer, TransportError};
use aeron_rs::concurrent::atomic_buffer::AtomicBuffer;
use aeron_rs::concurrent::logbuffer::buffer_claim::BufferClaim;
use aeron_rs::publication::Publication;

use super::error::result_to_transport_error;

/// Aeron-rs based implementation of [`AeronPublisher`].
///
/// This publisher wraps an aeron-rs [`Publication`] and implements the
/// `AeronPublisher` trait using pure Rust.
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
pub struct AeronRsPublisher {
    publication: Publication,
}

impl AeronRsPublisher {
    /// Creates a new `AeronRsPublisher` wrapping the given aeron-rs publication.
    pub fn new(publication: Publication) -> Self {
        AeronRsPublisher { publication }
    }
}

impl AeronPublisher for AeronRsPublisher {
    fn offer(&mut self, buffer: &[u8]) -> Result<i64, TransportError> {
        // aeron-rs offer takes AtomicBuffer, which needs a mutable slice
        // We need to copy to a mutable buffer since our API takes &[u8]
        // Clone: Required because aeron-rs AtomicBuffer needs &mut [u8] but our trait takes &[u8]
        let mut buffer_copy = buffer.to_vec();
        let atomic_buffer = AtomicBuffer::wrap_slice(&mut buffer_copy);

        // Call aeron-rs offer method
        let result = self.publication.offer(atomic_buffer);

        result_to_transport_error(result)
    }

    fn try_claim<'a>(&'a mut self, length: usize) -> Result<ClaimBuffer<'a>, TransportError> {
        // aeron-rs try_claim populates a BufferClaim
        let mut buffer_claim = BufferClaim::default();

        let result = self
            .publication
            .try_claim(length as i32, &mut buffer_claim);

        match result {
            Ok(position) => {
                // Get mutable access to the claimed buffer
                // aeron-rs BufferClaim provides buffer() method returning AtomicBuffer
                let buffer = buffer_claim.buffer();

                // Create a mutable slice from the AtomicBuffer
                // Safety: The buffer is valid while the BufferClaim is held
                let slice = unsafe {
                    std::slice::from_raw_parts_mut(
                        buffer.buffer(),
                        buffer.capacity() as usize,
                    )
                };

                Ok(ClaimBuffer::new(slice, position as i64))
            }
            Err(e) => Err(super::error::aeron_error_to_transport_error(e)),
        }
    }
}
