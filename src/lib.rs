//! Aerofoil - Wingfoil Aeron transport adapters
//!
//! This library provides trait-based abstractions for Aeron messaging that enable
//! zero-cost polymorphism across different Aeron client implementations. It supports
//! both Rusteron (C++ Aeron wrapper) and aeron-rs (pure Rust client) through
//! compile-time feature selection, with mock implementations for testing.
//!
//! # Design Philosophy
//!
//! - **Zero-cost abstraction**: Traits use static dispatch with no runtime overhead
//! - **Non-blocking**: All operations return immediately, critical for HFT systems
//! - **Zero-copy**: Direct buffer access where supported by the underlying client
//! - **Testable**: Mock implementations require no Aeron infrastructure
//!
//! # Architecture
//!
//! The library is organized around two core traits:
//!
//! - [`transport::AeronPublisher`] - Publishes messages to Aeron channels
//! - [`transport::AeronSubscriber`] - Receives messages from Aeron channels
//!
//! These traits are implemented by:
//!
//! - **Rusteron adapter** (feature `rusteron`) - Wraps the official C++ Aeron client
//! - **Aeron-rs adapter** (feature `aeron-rs`) - Pure Rust Aeron implementation
//! - **Mock implementations** - In-memory testing without Aeron
//!
//! # Feature Flags
//!
//! Backend selection is controlled via Cargo features (mutually exclusive):
//!
//! - `rusteron` (default) - Use Rusteron C++ wrapper (requires C++ toolchain)
//! - `aeron-rs` - Use pure Rust aeron-rs client
//!
//! To use aeron-rs instead of the default:
//!
//! ```toml
//! [dependencies]
//! aerofoil = { version = "0.1", default-features = false, features = ["aeron-rs"] }
//! ```
//!
//! # Examples
//!
//! ## Publishing Messages
//!
//! ```ignore
//! use aerofoil::transport::{AeronPublisher, MockPublisher};
//!
//! let mut publisher = MockPublisher::new();
//!
//! // Simple publish (copies data)
//! let position = publisher.offer(b"hello world")?;
//!
//! // Zero-copy publish
//! let mut claim = publisher.try_claim(256)?;
//! claim[0..5].copy_from_slice(b"hello");
//! // Automatically committed on drop
//! ```
//!
//! ## Receiving Messages
//!
//! ```ignore
//! use aerofoil::transport::{AeronSubscriber, MockSubscriber};
//!
//! let mut subscriber = MockSubscriber::new();
//! subscriber.inject_message(b"test message".to_vec());
//!
//! subscriber.poll(|fragment| {
//!     println!("Received: {:?}", fragment.as_ref());
//!     Ok(())
//! })?;
//! ```
//!
//! ## Generic Code with Trait Bounds
//!
//! ```ignore
//! use aerofoil::transport::{AeronPublisher, TransportError};
//!
//! fn send_heartbeat<P: AeronPublisher>(
//!     publisher: &mut P
//! ) -> Result<i64, TransportError> {
//!     publisher.offer(b"HEARTBEAT")
//! }
//!
//! // Works with any backend: Rusteron, aeron-rs, or mocks
//! ```
//!
//! # Testing
//!
//! Use [`MockPublisher`] and
//! [`MockSubscriber`] for unit tests:
//!
//! ```
//! use aerofoil::transport::{MockPublisher, AeronPublisher};
//!
//! #[test]
//! fn test_publishing() {
//!     let mut publisher = MockPublisher::new();
//!     publisher.offer(b"test").unwrap();
//!
//!     let messages = publisher.published_messages();
//!     assert_eq!(messages.len(), 1);
//!     assert_eq!(messages[0], b"test");
//! }
//! ```
//!
//! For advanced mocking with expectations, use mockall:
//!
//! ```ignore
//! use aerofoil::transport::MockAeronPublisher;
//! use mockall::predicate::*;
//!
//! let mut mock = MockAeronPublisher::new();
//! mock.expect_offer()
//!     .with(eq(b"expected message"))
//!     .times(1)
//!     .returning(|_| Ok(0));
//! ```

pub mod transport;

pub use transport::{
    AeronPublisher, AeronSubscriber, ClaimBuffer, FragmentBuffer, FragmentHeader, MockPublisher,
    MockSubscriber, TransportError,
};
