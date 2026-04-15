//! Type-safe builders for Aeron channel URIs.
//!
//! Constructing Aeron channel URIs by hand is error-prone — typos in the
//! `aeron:udp?endpoint=...` syntax are silently accepted by the media driver
//! and surface only as a non-connecting publication. The [`ChannelUri`]
//! helpers below produce the canonical strings for the most common channel
//! shapes used by `aerofoil`-based applications.

/// Builders for Aeron channel URI strings.
///
/// All constructors return `String` rather than a wrapper type — Aeron's
/// public API expects a `&str`, and adding a wrapper would force callers to
/// `.as_str()` everywhere with no extra safety. The associated functions
/// document the exact format they emit.
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
    /// # Examples
    ///
    /// ```
    /// # use aerofoil::transport::ChannelUri;
    /// assert_eq!(
    ///     ChannelUri::udp("127.0.0.1:40123"),
    ///     "aeron:udp?endpoint=127.0.0.1:40123"
    /// );
    /// ```
    pub fn udp(endpoint: &str) -> String {
        format!("aeron:udp?endpoint={endpoint}")
    }

    /// Returns an MDC publication channel URI:
    /// `aeron:udp?control={control}|control-mode=dynamic`.
    ///
    /// Use this for the publisher side of a Multi-Destination-Cast stream.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aerofoil::transport::ChannelUri;
    /// assert_eq!(
    ///     ChannelUri::mdc_publication("127.0.0.1:40456"),
    ///     "aeron:udp?control=127.0.0.1:40456|control-mode=dynamic"
    /// );
    /// ```
    pub fn mdc_publication(control: &str) -> String {
        format!("aeron:udp?control={control}|control-mode=dynamic")
    }

    /// Returns an MDC subscription channel URI:
    /// `aeron:udp?endpoint={endpoint}|control={control}|control-mode=dynamic`.
    ///
    /// Use this for the subscriber side of a Multi-Destination-Cast stream.
    ///
    /// # Examples
    ///
    /// ```
    /// # use aerofoil::transport::ChannelUri;
    /// assert_eq!(
    ///     ChannelUri::mdc_subscription("127.0.0.1:40789", "127.0.0.1:40456"),
    ///     "aeron:udp?endpoint=127.0.0.1:40789|control=127.0.0.1:40456|control-mode=dynamic"
    /// );
    /// ```
    pub fn mdc_subscription(endpoint: &str, control: &str) -> String {
        format!("aeron:udp?endpoint={endpoint}|control={control}|control-mode=dynamic")
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
            ChannelUri::udp("127.0.0.1:40123"),
            "aeron:udp?endpoint=127.0.0.1:40123"
        );
    }

    #[test]
    fn given_channel_uri_when_mdc_publication_then_formats_control_dynamic() {
        assert_eq!(
            ChannelUri::mdc_publication("127.0.0.1:40456"),
            "aeron:udp?control=127.0.0.1:40456|control-mode=dynamic"
        );
    }

    #[test]
    fn given_channel_uri_when_mdc_subscription_then_formats_endpoint_and_control() {
        assert_eq!(
            ChannelUri::mdc_subscription("127.0.0.1:40789", "127.0.0.1:40456"),
            "aeron:udp?endpoint=127.0.0.1:40789|control=127.0.0.1:40456|control-mode=dynamic"
        );
    }
}
