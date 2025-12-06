//! Rusteron implementation of AeronPublisher trait.

use crate::transport::{AeronPublisher, ClaimBuffer, TransportError};
use rusteron_client::AeronPublication;

use super::error::result_to_transport_error;

/// Rusteron-based implementation of [`AeronPublisher`].
///
/// This publisher wraps a Rusteron [`AeronPublication`] and implements the
/// `AeronPublisher` trait.
pub struct RusteronPublisher {
    publication: AeronPublication,
}

impl RusteronPublisher {
    /// Creates a new `RusteronPublisher` wrapping the given Rusteron publication.
    pub fn new(publication: AeronPublication) -> Self {
        RusteronPublisher { publication }
    }
}

impl AeronPublisher for RusteronPublisher {
    fn offer(&mut self, buffer: &[u8]) -> Result<i64, TransportError> {
        // Call Rusteron's offer method
        // Type annotation required because offer is generic over the reserved value handler
        let result = self
            .publication
            .offer::<rusteron_client::AeronReservedValueSupplierLogger>(buffer, None);

        result_to_transport_error(result)
    }

    fn offer_mut(&mut self, buffer: &mut [u8]) -> Result<i64, TransportError> {
        // Rusteron accepts &[u8], and &mut [u8] coerces to &[u8]
        let result = self
            .publication
            .offer::<rusteron_client::AeronReservedValueSupplierLogger>(buffer, None);

        result_to_transport_error(result)
    }

    fn try_claim<'a>(&'a mut self, _length: usize) -> Result<ClaimBuffer<'a>, TransportError> {
        // TODO: Implement try_claim using Rusteron's AeronBufferClaim
        // This requires investigating how to safely expose the buffer from AeronBufferClaim
        Err(TransportError::Invalid(
            "try_claim not yet implemented for Rusteron".to_string(),
        ))
    }
}
