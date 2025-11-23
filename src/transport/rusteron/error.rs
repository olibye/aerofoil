//! Error handling and conversion for Rusteron adapter.
//!
//! This module maps Rusteron-specific errors to the common [`TransportError`](crate::transport::TransportError)
//! type, providing unified error handling across all transport implementations.

use crate::transport::TransportError;

/// Converts Rusteron result codes to TransportError.
///
/// Rusteron's `offer` and `try_claim` methods return i64 values where:
/// - Positive values indicate success (stream position)
/// - Negative values indicate specific error conditions
///
/// Common error codes (from Aeron C client):
/// - `-1` (`NOT_CONNECTED`): Publication not connected
/// - `-2` (`BACK_PRESSURED`): Offer failed due to back pressure
/// - `-3` (`ADMIN_ACTION`): Action is an administration action
/// - `-4` (`PUBLICATION_CLOSED`): Publication has been closed
/// - `-5` (`MAX_POSITION_EXCEEDED`): Offer failed due to max position being reached
///
/// # Example
///
/// ```ignore
/// let result = publication.offer(buffer, None);
/// let position = result_to_transport_error(result)?;
/// ```
pub fn result_to_transport_error(result: i64) -> Result<i64, TransportError> {
    if result >= 0 {
        Ok(result)
    } else {
        match result {
            -1 => Err(TransportError::NotConnected),
            -2 => Err(TransportError::BackPressure),
            -4 => Err(TransportError::NotConnected), // PUBLICATION_CLOSED
            code => Err(TransportError::Backend(format!(
                "Rusteron error code: {}",
                code
            ))),
        }
    }
}

/// Converts Rusteron AeronCError to TransportError.
///
/// # Design Decision: Error Conversion
///
/// We convert rusteron_client::AeronCError to our TransportError type to:
/// - **Provide unified API**: Consistent error handling across all backends
/// - **Enable `?` operator**: Idiomatic Rust error propagation
/// - **Preserve error details**: Original error code and message retained
///
/// # Example
///
/// ```ignore
/// subscription.poll(...).map_err(aeron_error_to_transport_error)?;
/// ```
pub fn aeron_error_to_transport_error(err: rusteron_client::AeronCError) -> TransportError {
    // AeronCError contains error code and message
    // Map common codes to specific TransportError variants
    let error_msg = format!("Rusteron error: {:?}", err);

    // Try to infer error type from message if possible
    // For now, map all to Backend variant with full error details
    TransportError::Backend(error_msg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_result() {
        // Given: Successful operation result
        let result = 12345i64;

        // When: Converting to TransportError
        let position = result_to_transport_error(result);

        // Then: Returns Ok with position
        assert_eq!(position.unwrap(), 12345);
    }

    #[test]
    fn test_back_pressure() {
        // Given: Back pressure error code
        let result = -2i64;

        // When: Converting to TransportError
        let err = result_to_transport_error(result);

        // Then: Returns BackPressure variant
        assert!(matches!(err, Err(TransportError::BackPressure)));
    }

    #[test]
    fn test_not_connected() {
        // Given: Not connected error code
        let result = -1i64;

        // When: Converting to TransportError
        let err = result_to_transport_error(result);

        // Then: Returns NotConnected variant
        assert!(matches!(err, Err(TransportError::NotConnected)));
    }

    #[test]
    fn test_publication_closed() {
        // Given: Publication closed error code
        let result = -4i64;

        // When: Converting to TransportError
        let err = result_to_transport_error(result);

        // Then: Returns NotConnected variant
        assert!(matches!(err, Err(TransportError::NotConnected)));
    }

    #[test]
    fn test_unknown_error() {
        // Given: Unknown error code
        let result = -99i64;

        // When: Converting to TransportError
        let err = result_to_transport_error(result);

        // Then: Returns Backend variant with error code
        match err {
            Err(TransportError::Backend(msg)) => {
                assert!(msg.contains("-99"));
            }
            _ => panic!("Expected Backend error"),
        }
    }
}
