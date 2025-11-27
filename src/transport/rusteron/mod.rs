//! Rusteron adapter for Aeron transport traits.
//!
//! This module provides implementations of the transport traits using the
//! Rusteron library, which wraps the official C++ Aeron client.

mod error;
mod publisher;
mod subscriber;

pub use error::result_to_transport_error;
pub use publisher::RusteronPublisher;
pub use subscriber::RusteronSubscriber;
