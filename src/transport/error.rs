//! Transport error types.

use std::fmt;

/// Unified error type for transport operations.
///
/// This error type provides a consistent interface across different Aeron
/// client implementations (Rusteron, aeron-rs) and enables uniform error
/// handling in application code.
#[derive(Debug)]
#[non_exhaustive]
pub enum TransportError {
    /// Back-pressure condition: buffer is full, retry later.
    ///
    /// This is returned when the Aeron publication buffer cannot accept
    /// new messages. In HFT systems, this typically indicates the need
    /// for flow control or rate limiting.
    BackPressure,

    /// Connection-related errors (not connected, closed, etc.)
    Connection(String),

    /// Backend-specific error with descriptive message.
    ///
    /// Contains the error details from the underlying Aeron client.
    Backend(String),

    /// Invalid operation or parameters.
    Invalid(String),
}

impl fmt::Display for TransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransportError::BackPressure => write!(f, "Back-pressure: buffer full"),
            TransportError::Connection(msg) => write!(f, "Connection error: {}", msg),
            TransportError::Backend(msg) => write!(f, "Backend error: {}", msg),
            TransportError::Invalid(msg) => write!(f, "Invalid operation: {}", msg),
        }
    }
}

impl std::error::Error for TransportError {}
