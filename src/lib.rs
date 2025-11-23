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
//! - **Test implementations** - Simple trait implementations for testing (users implement as needed)
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
//! use aerofoil::transport::AeronPublisher;
//!
//! // With a real implementation (Rusteron or aeron-rs)
//! let mut publisher = create_publisher(); // Implementation-specific
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
//! use aerofoil::transport::AeronSubscriber;
//!
//! // With a real implementation (Rusteron or aeron-rs)
//! let mut subscriber = create_subscriber(); // Implementation-specific
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
//! For unit testing without Aeron, implement the traits with simple test doubles:
//!
//! ```ignore
//! use aerofoil::transport::{AeronPublisher, TransportError};
//!
//! struct TestPublisher {
//!     messages: Vec<Vec<u8>>,
//! }
//!
//! impl AeronPublisher for TestPublisher {
//!     fn offer(&mut self, buffer: &[u8]) -> Result<i64, TransportError> {
//!         self.messages.push(buffer.to_vec());
//!         Ok(self.messages.len() as i64 - 1)
//!     }
//!
//!     fn try_claim<'a>(&'a mut self, length: usize) -> Result<ClaimBuffer<'a>, TransportError> {
//!         // Implementation details...
//!         unimplemented!()
//!     }
//! }
//!
//! #[test]
//! fn test_my_code() {
//!     let mut publisher = TestPublisher { messages: Vec::new() };
//!     my_function_that_publishes(&mut publisher).unwrap();
//!     assert_eq!(publisher.messages.len(), 1);
//! }
//! ```
//!
//! The traits are designed to be easy to implement for testing purposes.

pub mod transport;

pub use transport::{
    AeronPublisher, AeronSubscriber, ClaimBuffer, FragmentBuffer, FragmentHeader, TransportError,
};
