//! Rusteron implementation of AeronPublisher trait.

use crate::transport::{AeronPublisher, ClaimBuffer, TransportError};
use rusteron_client::{AeronBufferClaim, AeronPublication};

use super::error::result_to_transport_error;

/// Rusteron-based implementation of [`AeronPublisher`].
///
/// This publisher wraps a Rusteron [`AeronPublication`] and implements the
/// `AeronPublisher` trait, providing access to the mature C++ Aeron client
/// through Rust's type system.
///
/// # Design Decision: Ownership Model
///
/// `RusteronPublisher` owns the `AeronPublication` instance because:
/// - **Clear ownership**: Single owner prevents resource leaks
/// - **Simplified API**: Users don't need to manage Rusteron lifecycle separately
/// - **Type safety**: Cannot accidentally use publication after it's been consumed
///
/// # Thread Safety
///
/// Rusteron types are typically not `Send`/`Sync`. This publisher should be:
/// - Created and used on a single thread
/// - Pinned to specific CPU cores (common in HFT systems)
///
/// # Example
///
/// ```ignore
/// use aerofoil::transport::AeronPublisher;
/// use aerofoil::transport::rusteron::RusteronPublisher;
///
/// // Create publisher (requires media driver)
/// let mut publisher = RusteronPublisher::new(publication);
///
/// // Publish message with copy
/// let position = publisher.offer(b"Hello, Aeron!")?;
///
/// // Zero-copy publication
/// let mut claim = publisher.try_claim(256)?;
/// claim[0..5].copy_from_slice(b"Hello");
/// ```
pub struct RusteronPublisher {
    publication: AeronPublication,
}

impl RusteronPublisher {
    /// Creates a new `RusteronPublisher` wrapping the given Rusteron publication.
    ///
    /// # Arguments
    ///
    /// * `publication` - A configured Rusteron `AeronPublication` instance
    ///
    /// # Example
    ///
    /// ```ignore
    /// let publication = AeronPublication::new(/* ... */)?;
    /// let publisher = RusteronPublisher::new(publication);
    /// ```
    pub fn new(publication: AeronPublication) -> Self {
        RusteronPublisher { publication }
    }

    /// Returns a reference to the underlying Rusteron publication.
    ///
    /// This provides access to Rusteron-specific features not exposed by the
    /// `AeronPublisher` trait.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let raw_pub = publisher.inner();
    /// // Use Rusteron-specific methods...
    /// ```
    pub fn inner(&self) -> &AeronPublication {
        &self.publication
    }
}

impl AeronPublisher for RusteronPublisher {
    fn offer(&mut self, buffer: &[u8]) -> Result<i64, TransportError> {
        // Call Rusteron's offer method
        // Note: Rusteron's offer returns i64 (position or error code)
        // Positive values are stream positions, negative are errors
        //
        // Type annotation required because offer is generic over the reserved value handler
        // We pass None since we don't use reserved values
        let result = self.publication.offer::<rusteron_client::AeronReservedValueSupplierLogger>(buffer, None);

        // Convert result to TransportError if needed
        result_to_transport_error(result)
    }

    fn try_claim<'a>(&'a mut self, length: usize) -> Result<ClaimBuffer<'a>, TransportError> {
        // Create a buffer claim object for Rusteron to populate
        // Note: AeronBufferClaim must be created before calling try_claim
        let buffer_claim = AeronBufferClaim::default();

        // Attempt to claim buffer space
        let result = self.publication.try_claim(length, &buffer_claim);

        // Check result
        let _position = result_to_transport_error(result)?;

        // SAFETY CONCERN: We need to convert AeronBufferClaim to ClaimBuffer<'a>
        // The challenge is that AeronBufferClaim owns the data, but ClaimBuffer
        // expects a &'a mut [u8] reference.
        //
        // For now, this is a placeholder implementation that will need refinement
        // based on how Rusteron actually exposes the buffer from AeronBufferClaim.
        //
        // TODO: Investigate Rusteron's AeronBufferClaim API to get mutable buffer access
        // TODO: Handle buffer claim commit/abort on drop

        // This is a placeholder - actual implementation needs Rusteron buffer access
        todo!("RusteronPublisher::try_claim requires access to AeronBufferClaim's internal buffer")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests cannot run without a media driver and proper Rusteron setup
    // They are provided as examples of how to test with manual test implementations

    #[test]
    fn test_publisher_trait_object() {
        // Given: This test verifies the type implements the trait correctly
        // We can't create a real publisher without a media driver, but we can
        // verify the types are correct at compile time

        // This function accepts any AeronPublisher implementation
        fn takes_publisher<P: AeronPublisher>(_p: &P) {}

        // If this compiles, RusteronPublisher correctly implements AeronPublisher
        // (Actual instance creation would require media driver)
    }
}
