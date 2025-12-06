//! Error conversion for aeron-rs results.

use crate::transport::TransportError;

/// Converts aeron-rs AeronError to TransportError.
///
/// Aeron-rs uses Result<T, AeronError> for error handling.
/// This maps aeron-rs error variants to our unified TransportError.
pub fn aeron_error_to_transport_error(err: aeron_rs::utils::errors::AeronError) -> TransportError {
    use aeron_rs::utils::errors::AeronError;

    match err {
        AeronError::BackPressured => TransportError::BackPressure,
        AeronError::NotConnected => TransportError::Connection("Not connected".to_string()),
        AeronError::PublicationClosed => TransportError::Connection("Publication closed".to_string()),
        AeronError::AdminAction => {
            // AdminAction means the operation should be retried - treat as back-pressure
            TransportError::BackPressure
        }
        AeronError::MaxPositionExceeded => {
            TransportError::Invalid("Max position exceeded".to_string())
        }
        other => TransportError::Backend(format!("aeron-rs error: {:?}", other)),
    }
}

/// Converts aeron-rs Result to TransportError Result.
///
/// # Arguments
///
/// * `result` - The Result from an aeron-rs operation
///
/// # Returns
///
/// * `Ok(position)` - Success with stream position
/// * `Err(TransportError)` - Mapped error condition
pub fn result_to_transport_error(
    result: Result<u64, aeron_rs::utils::errors::AeronError>,
) -> Result<i64, TransportError> {
    result
        .map(|pos| pos as i64)
        .map_err(aeron_error_to_transport_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aeron_rs::utils::errors::AeronError;

    #[test]
    fn given_back_pressured_error_when_converting_then_returns_back_pressure() {
        // Given: aeron-rs BackPressured error
        let err = AeronError::BackPressured;

        // When: Converting to TransportError
        let result = aeron_error_to_transport_error(err);

        // Then: Should return BackPressure
        matches!(result, TransportError::BackPressure);
    }

    #[test]
    fn given_not_connected_error_when_converting_then_returns_connection_error() {
        // Given: aeron-rs NotConnected error
        let err = AeronError::NotConnected;

        // When: Converting to TransportError
        let result = aeron_error_to_transport_error(err);

        // Then: Should return Connection error
        matches!(result, TransportError::Connection(_));
    }

    #[test]
    fn given_publication_closed_error_when_converting_then_returns_connection_error() {
        // Given: aeron-rs PublicationClosed error
        let err = AeronError::PublicationClosed;

        // When: Converting to TransportError
        let result = aeron_error_to_transport_error(err);

        // Then: Should return Connection error
        matches!(result, TransportError::Connection(_));
    }

    #[test]
    fn given_admin_action_error_when_converting_then_returns_back_pressure() {
        // Given: aeron-rs AdminAction error (retry-able)
        let err = AeronError::AdminAction;

        // When: Converting to TransportError
        let result = aeron_error_to_transport_error(err);

        // Then: Should return BackPressure (indicating retry)
        matches!(result, TransportError::BackPressure);
    }

    #[test]
    fn given_success_result_when_converting_then_returns_position() {
        // Given: A successful result with position
        let result: Result<u64, AeronError> = Ok(12345u64);

        // When: Converting to TransportError result
        let converted = result_to_transport_error(result);

        // Then: Should return Ok with position as i64
        assert!(converted.is_ok());
        assert_eq!(converted.unwrap(), 12345i64);
    }
}
