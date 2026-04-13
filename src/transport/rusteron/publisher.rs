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

impl std::fmt::Debug for RusteronPublisher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RusteronPublisher").finish_non_exhaustive()
    }
}

impl RusteronPublisher {
    /// Creates a new `RusteronPublisher` wrapping the given Rusteron publication.
    pub fn new(publication: AeronPublication) -> Self {
        RusteronPublisher { publication }
    }
}

impl AeronPublisher for RusteronPublisher {
    fn is_connected(&self) -> bool {
        self.publication.is_connected()
    }

    fn is_closed(&self) -> bool {
        self.publication.is_closed()
    }

    fn offer(&mut self, buffer: &[u8]) -> Result<i64, TransportError> {
        // Call Rusteron's offer method
        // Type annotation required because offer is generic over the reserved value handler
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
