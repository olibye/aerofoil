//! Named publisher / subscriber discovery.
//!
//! Provides a process-global, in-memory registry that maps logical names to
//! `(channel, stream_id)` tuples. Applications register their endpoints at
//! startup and then resolve publishers/subscribers by name rather than
//! threading channel strings and magic stream IDs through every call site.
//!
//! The registry is intentionally **in-process only** — file or network
//! discovery is deferred (see story 9.3 Dev Notes).

use std::collections::HashMap;
use std::fmt;
use std::sync::{Mutex, OnceLock};

use super::{AeronPublisher, AeronSubscriber};

/// Errors returned by named discovery operations.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DiscoveryError {
    /// No publisher or subscriber has been registered under the given name.
    Unknown(String),
    /// Registration rejected because the name is empty.
    EmptyName,
}

impl fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiscoveryError::Unknown(name) => write!(f, "unknown discovery name: {name}"),
            DiscoveryError::EmptyName => write!(f, "discovery name must not be empty"),
        }
    }
}

impl std::error::Error for DiscoveryError {}

type Registry = OnceLock<Mutex<HashMap<String, (String, i32)>>>;

static PUB_REGISTRY: Registry = OnceLock::new();
static SUB_REGISTRY: Registry = OnceLock::new();

fn registry(reg: &'static Registry) -> &'static Mutex<HashMap<String, (String, i32)>> {
    reg.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Registers a publisher endpoint under `name`.
///
/// Re-registering an existing name **overwrites** the prior entry. This is
/// deterministic and matches the typical "configuration applied last wins"
/// expectation from application startup code.
///
/// # Errors
///
/// Returns [`DiscoveryError::EmptyName`] if `name` is empty.
pub fn register_pub(name: &str, channel: String, stream_id: i32) -> Result<(), DiscoveryError> {
    if name.is_empty() {
        return Err(DiscoveryError::EmptyName);
    }
    let mut map = registry(&PUB_REGISTRY)
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    map.insert(name.to_string(), (channel, stream_id));
    Ok(())
}

/// Registers a subscriber endpoint under `name`.
///
/// Re-registering an existing name **overwrites** the prior entry.
///
/// # Errors
///
/// Returns [`DiscoveryError::EmptyName`] if `name` is empty.
pub fn register_sub(name: &str, channel: String, stream_id: i32) -> Result<(), DiscoveryError> {
    if name.is_empty() {
        return Err(DiscoveryError::EmptyName);
    }
    let mut map = registry(&SUB_REGISTRY)
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    map.insert(name.to_string(), (channel, stream_id));
    Ok(())
}

/// Returns the registered `(channel, stream_id)` for a publisher name, if any.
pub fn lookup_pub(name: &str) -> Option<(String, i32)> {
    let map = registry(&PUB_REGISTRY)
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    map.get(name).cloned()
}

/// Returns the registered `(channel, stream_id)` for a subscriber name, if any.
pub fn lookup_sub(name: &str) -> Option<(String, i32)> {
    let map = registry(&SUB_REGISTRY)
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    map.get(name).cloned()
}

/// Validates that `name` is a registered publisher and returns the supplied
/// publisher unchanged.
///
/// This is a pass-through guard intended for application startup wiring:
/// callers construct the publisher however their backend requires and then
/// hand it through `aeron_pub_named` to assert that the logical name is
/// known to the discovery layer.
pub fn aeron_pub_named<P: AeronPublisher>(name: &str, publisher: P) -> Result<P, DiscoveryError> {
    if lookup_pub(name).is_some() {
        Ok(publisher)
    } else {
        Err(DiscoveryError::Unknown(name.to_string()))
    }
}

/// Validates that `name` is a registered subscriber and returns the supplied
/// subscriber unchanged.
pub fn aeron_sub_discover<S: AeronSubscriber>(
    name: &str,
    subscriber: S,
) -> Result<S, DiscoveryError> {
    if lookup_sub(name).is_some() {
        Ok(subscriber)
    } else {
        Err(DiscoveryError::Unknown(name.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::{ClaimBuffer, FragmentBuffer, TransportError};

    struct MockPublisher;

    impl AeronPublisher for MockPublisher {
        fn offer(&mut self, _buffer: &[u8]) -> Result<i64, TransportError> {
            Ok(0)
        }

        fn try_claim<'a>(&'a mut self, _length: usize) -> Result<ClaimBuffer<'a>, TransportError> {
            Err(TransportError::Invalid("mock".to_string()))
        }
    }

    struct MockSubscriber;

    impl AeronSubscriber for MockSubscriber {
        fn poll<F>(&mut self, _handler: F) -> Result<usize, TransportError>
        where
            F: FnMut(&FragmentBuffer) -> Result<(), TransportError>,
        {
            Ok(0)
        }
    }

    #[test]
    fn given_registered_pub_when_aeron_pub_named_then_returns_publisher() {
        register_pub("test_pub_round_trip", "aeron:ipc".to_string(), 42).unwrap();
        let result = aeron_pub_named("test_pub_round_trip", MockPublisher);
        assert!(result.is_ok());
        assert_eq!(
            lookup_pub("test_pub_round_trip"),
            Some(("aeron:ipc".to_string(), 42))
        );
    }

    #[test]
    fn given_unregistered_name_when_aeron_pub_named_then_returns_unknown() {
        let result = aeron_pub_named("test_pub_unregistered_xyz", MockPublisher);
        assert_eq!(
            result.err(),
            Some(DiscoveryError::Unknown(
                "test_pub_unregistered_xyz".to_string()
            ))
        );
    }

    #[test]
    fn given_registered_sub_when_aeron_sub_discover_then_returns_subscriber() {
        register_sub(
            "test_sub_round_trip",
            "aeron:udp?endpoint=127.0.0.1:40000".to_string(),
            7,
        )
        .unwrap();
        let result = aeron_sub_discover("test_sub_round_trip", MockSubscriber);
        assert!(result.is_ok());
        assert_eq!(
            lookup_sub("test_sub_round_trip"),
            Some(("aeron:udp?endpoint=127.0.0.1:40000".to_string(), 7))
        );
    }

    #[test]
    fn given_unregistered_name_when_aeron_sub_discover_then_returns_unknown() {
        let result = aeron_sub_discover("test_sub_unregistered_xyz", MockSubscriber);
        assert_eq!(
            result.err(),
            Some(DiscoveryError::Unknown(
                "test_sub_unregistered_xyz".to_string()
            ))
        );
    }

    #[test]
    fn given_registered_pub_when_re_registered_then_overwrites() {
        register_pub("test_pub_overwrite", "aeron:ipc".to_string(), 1).unwrap();
        register_pub(
            "test_pub_overwrite",
            "aeron:udp?endpoint=127.0.0.1:1".to_string(),
            2,
        )
        .unwrap();
        assert_eq!(
            lookup_pub("test_pub_overwrite"),
            Some(("aeron:udp?endpoint=127.0.0.1:1".to_string(), 2))
        );
    }

    #[test]
    fn given_registered_sub_when_re_registered_then_overwrites() {
        register_sub("test_sub_overwrite", "aeron:ipc".to_string(), 1).unwrap();
        register_sub(
            "test_sub_overwrite",
            "aeron:udp?endpoint=127.0.0.1:2".to_string(),
            9,
        )
        .unwrap();
        assert_eq!(
            lookup_sub("test_sub_overwrite"),
            Some(("aeron:udp?endpoint=127.0.0.1:2".to_string(), 9))
        );
    }

    #[test]
    fn given_empty_name_when_register_pub_then_returns_empty_name_error() {
        let err = register_pub("", "aeron:ipc".to_string(), 1).unwrap_err();
        assert_eq!(err, DiscoveryError::EmptyName);
    }

    #[test]
    fn given_empty_name_when_register_sub_then_returns_empty_name_error() {
        let err = register_sub("", "aeron:ipc".to_string(), 1).unwrap_err();
        assert_eq!(err, DiscoveryError::EmptyName);
    }

    #[test]
    fn given_discovery_error_when_display_then_includes_name() {
        let err = DiscoveryError::Unknown("foo".to_string());
        assert_eq!(format!("{err}"), "unknown discovery name: foo");
    }

    #[test]
    fn given_empty_name_error_when_display_then_describes_issue() {
        let err = DiscoveryError::EmptyName;
        assert_eq!(format!("{err}"), "discovery name must not be empty");
    }
}
