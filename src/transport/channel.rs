//! Type-safe builders for Aeron channel URIs.
//!
//! Constructing Aeron channel URIs by hand is error-prone — typos in the
//! `aeron:udp?endpoint=...` syntax are silently accepted by the media driver
//! and surface only as a non-connecting publication. The [`ChannelUri`]
//! helpers below produce the canonical strings for the most common channel
//! shapes used by `aerofoil`-based applications.

use super::TransportError;

/// Characters that are Aeron URI structural separators and must not appear
/// in endpoint or control address values.
const AERON_URI_RESERVED: &[char] = &['|', '?'];

/// Validates that an Aeron URI parameter value is non-empty and contains no
/// reserved separator characters.
fn validate_param(label: &str, value: &str) -> Result<(), TransportError> {
    if value.is_empty() {
        return Err(TransportError::Invalid(format!(
            "{label} must not be empty"
        )));
    }
    if let Some(ch) = value.chars().find(|c| AERON_URI_RESERVED.contains(c)) {
        return Err(TransportError::Invalid(format!(
            "{label} contains reserved character '{ch}'"
        )));
    }
    Ok(())
}

/// Builders for Aeron channel URI strings.
///
/// Parameterised constructors return `Result<String, TransportError>` and
/// reject empty values or values containing Aeron URI separator characters
/// (`|`, `?`) that would corrupt the URI structure. These checks run at
/// startup, not on the hot path.
#[derive(Debug)]
pub struct ChannelUri;

impl ChannelUri {
    /// Returns the IPC channel URI: `aeron:ipc`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aerofoil::transport::ChannelUri;
    /// assert_eq!(ChannelUri::ipc(), "aeron:ipc");
    /// ```
    pub fn ipc() -> String {
        "aeron:ipc".to_string()
    }

    /// Returns a UDP unicast channel URI: `aeron:udp?endpoint={endpoint}`.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError::Invalid`] if `endpoint` is empty or contains
    /// Aeron URI separator characters.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aerofoil::transport::ChannelUri;
    /// assert_eq!(
    ///     ChannelUri::udp("127.0.0.1:40123").unwrap(),
    ///     "aeron:udp?endpoint=127.0.0.1:40123"
    /// );
    /// ```
    pub fn udp(endpoint: &str) -> Result<String, TransportError> {
        validate_param("endpoint", endpoint)?;
        Ok(format!("aeron:udp?endpoint={endpoint}"))
    }

    /// Returns an MDC publication channel URI:
    /// `aeron:udp?control={control}|control-mode=dynamic`.
    ///
    /// Use this for the publisher side of a Multi-Destination-Cast stream.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError::Invalid`] if `control` is empty or contains
    /// Aeron URI separator characters.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aerofoil::transport::ChannelUri;
    /// assert_eq!(
    ///     ChannelUri::mdc_publication("127.0.0.1:40456").unwrap(),
    ///     "aeron:udp?control=127.0.0.1:40456|control-mode=dynamic"
    /// );
    /// ```
    pub fn mdc_publication(control: &str) -> Result<String, TransportError> {
        validate_param("control", control)?;
        Ok(format!("aeron:udp?control={control}|control-mode=dynamic"))
    }

    /// Returns an MDC subscription channel URI:
    /// `aeron:udp?endpoint={endpoint}|control={control}|control-mode=dynamic`.
    ///
    /// Use this for the subscriber side of a Multi-Destination-Cast stream.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError::Invalid`] if either parameter is empty or
    /// contains Aeron URI separator characters.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aerofoil::transport::ChannelUri;
    /// assert_eq!(
    ///     ChannelUri::mdc_subscription("127.0.0.1:40789", "127.0.0.1:40456").unwrap(),
    ///     "aeron:udp?endpoint=127.0.0.1:40789|control=127.0.0.1:40456|control-mode=dynamic"
    /// );
    /// ```
    pub fn mdc_subscription(endpoint: &str, control: &str) -> Result<String, TransportError> {
        validate_param("endpoint", endpoint)?;
        validate_param("control", control)?;
        Ok(format!(
            "aeron:udp?endpoint={endpoint}|control={control}|control-mode=dynamic"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_channel_uri_when_ipc_then_returns_aeron_ipc() {
        assert_eq!(ChannelUri::ipc(), "aeron:ipc");
    }

    #[test]
    fn given_channel_uri_when_udp_then_formats_endpoint() {
        assert_eq!(
            ChannelUri::udp("127.0.0.1:40123").unwrap(),
            "aeron:udp?endpoint=127.0.0.1:40123"
        );
    }

    #[test]
    fn given_channel_uri_when_mdc_publication_then_formats_control_dynamic() {
        assert_eq!(
            ChannelUri::mdc_publication("127.0.0.1:40456").unwrap(),
            "aeron:udp?control=127.0.0.1:40456|control-mode=dynamic"
        );
    }

    #[test]
    fn given_channel_uri_when_mdc_subscription_then_formats_endpoint_and_control() {
        assert_eq!(
            ChannelUri::mdc_subscription("127.0.0.1:40789", "127.0.0.1:40456").unwrap(),
            "aeron:udp?endpoint=127.0.0.1:40789|control=127.0.0.1:40456|control-mode=dynamic"
        );
    }

    #[test]
    fn given_channel_uri_when_udp_empty_endpoint_then_returns_error() {
        let err = ChannelUri::udp("").unwrap_err();
        assert!(matches!(err, TransportError::Invalid(_)));
    }

    #[test]
    fn given_channel_uri_when_udp_pipe_in_endpoint_then_returns_error() {
        let err = ChannelUri::udp("host|control-mode=dynamic").unwrap_err();
        assert!(matches!(err, TransportError::Invalid(_)));
    }

    #[test]
    fn given_channel_uri_when_udp_question_mark_in_endpoint_then_returns_error() {
        let err = ChannelUri::udp("host?evil=1").unwrap_err();
        assert!(matches!(err, TransportError::Invalid(_)));
    }

    #[test]
    fn given_channel_uri_when_mdc_publication_empty_control_then_returns_error() {
        let err = ChannelUri::mdc_publication("").unwrap_err();
        assert!(matches!(err, TransportError::Invalid(_)));
    }

    #[test]
    fn given_channel_uri_when_mdc_subscription_empty_endpoint_then_returns_error() {
        let err = ChannelUri::mdc_subscription("", "127.0.0.1:40456").unwrap_err();
        assert!(matches!(err, TransportError::Invalid(_)));
    }

    #[test]
    fn given_channel_uri_when_mdc_subscription_empty_control_then_returns_error() {
        let err = ChannelUri::mdc_subscription("127.0.0.1:40789", "").unwrap_err();
        assert!(matches!(err, TransportError::Invalid(_)));
    }
}
