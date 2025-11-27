//! Error conversion for Rusteron results.

use crate::transport::TransportError;

/// Converts Rusteron result codes to TransportError.
///
/// Rusteron methods return i64 where:
/// - Positive values indicate success (stream position)
/// - Negative values indicate errors:
///   - -1: Not connected
///   - -2: Back pressure
///   - -4: Publication closed
///
/// # Arguments
///
/// * `result` - The i64 result from a Rusteron operation
///
/// # Returns
///
/// * `Ok(position)` - Success with stream position
/// * `Err(TransportError)` - Mapped error condition
pub fn result_to_transport_error(result: i64) -> Result<i64, TransportError> {
    match result {
        pos if pos >= 0 => Ok(pos),
        -2 => Err(TransportError::BackPressure),
        -1 => Err(TransportError::Connection(
            "Not connected".to_string(),
        )),
        -4 => Err(TransportError::Connection(
            "Publication closed".to_string(),
        )),
        code => Err(TransportError::Backend(format!(
            "Rusteron error code: {}",
            code
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_result() {
        // Given: A positive result indicating stream position
        let result = 12345i64;

        // When: Converting to TransportError
        let converted = result_to_transport_error(result);

        // Then: Should return Ok with the position
        assert!(converted.is_ok());
        assert_eq!(converted.unwrap(), 12345);
    }

    #[test]
    fn test_back_pressure() {
        // Given: Result code -2 indicating back pressure
        let result = -2i64;

        // When: Converting to TransportError
        let converted = result_to_transport_error(result);

        // Then: Should return BackPressure error
        assert!(converted.is_err());
        matches!(converted.unwrap_err(), TransportError::BackPressure);
    }

    #[test]
    fn test_not_connected() {
        // Given: Result code -1 indicating not connected
        let result = -1i64;

        // When: Converting to TransportError
        let converted = result_to_transport_error(result);

        // Then: Should return Connection error
        assert!(converted.is_err());
        matches!(converted.unwrap_err(), TransportError::Connection(_));
    }
}
