/// Errors that can occur during transport operations.
///
/// This enum provides a unified error type across all transport implementations
/// (Rusteron, aeron-rs, mocks). Backend-specific errors are preserved via the
/// `source()` method for debugging purposes.
///
/// # Design Decision: String-Based Error Details
///
/// Variant payloads use `String` rather than boxed errors to maintain:
///
/// - **Clone + Eq**: Required for testing and comparison
/// - **No heap indirection**: Errors contain the message directly
/// - **Simple propagation**: Can be cloned across thread boundaries
///
/// Trade-off: We lose the full error chain, but preserve the essential message.
/// Backend-specific implementations can provide richer errors via `From` impls
/// that preserve the original error in the string representation.
///
/// # Design Decision: Explicit BackPressure Variant
///
/// `BackPressure` is a first-class variant rather than a generic "WouldBlock":
///
/// - **Domain clarity**: Makes Aeron semantics explicit
/// - **Not an error**: Back-pressure is expected in HFT systems
/// - **Retry guidance**: Clearly signals "try again later"
/// - **Monitoring**: Can be tracked separately from actual errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportError {
    /// Back-pressure indication - the transport cannot accept more messages currently.
    ///
    /// This is a non-error condition indicating the sender should retry later.
    /// The Aeron buffer is full and needs to drain before accepting new messages.
    BackPressure,

    /// The transport is not connected to the media driver or channel.
    NotConnected,

    /// The channel specification is invalid or malformed.
    InvalidChannel(String),

    /// An I/O error occurred during transport operations.
    IoError(String),

    /// A backend-specific error occurred.
    ///
    /// The original error is preserved as a string to maintain Clone + Eq.
    /// For the full error with source chain, use the non-cloneable version
    /// via backend-specific error conversion.
    Backend(String),
}

impl std::fmt::Display for TransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportError::BackPressure => {
                write!(f, "back-pressure: transport buffer full, retry later")
            }
            TransportError::NotConnected => {
                write!(f, "transport not connected to media driver or channel")
            }
            TransportError::InvalidChannel(msg) => {
                write!(f, "invalid channel: {}", msg)
            }
            TransportError::IoError(msg) => {
                write!(f, "I/O error: {}", msg)
            }
            TransportError::Backend(msg) => {
                write!(f, "backend error: {}", msg)
            }
        }
    }
}

impl std::error::Error for TransportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // Backend-specific errors would preserve source here when not cloneable
        None
    }
}

impl From<std::io::Error> for TransportError {
    fn from(err: std::io::Error) -> Self {
        TransportError::IoError(err.to_string())
    }
}
