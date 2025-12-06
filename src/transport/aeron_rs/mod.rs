//! Aeron-rs adapter for Aeron transport traits.
//!
//! This module provides implementations of the transport traits using the
//! aeron-rs library, a pure Rust Aeron client implementation.
//!
//! # Deployment Benefits
//!
//! - No C++ toolchain required
//! - Pure Rust memory safety guarantees
//! - Simpler cross-compilation
//!
//! # Trade-offs
//!
//! - Less mature than Rusteron (C++ wrapper)
//! - May have different performance characteristics
//!
//! See `openspec/project.md` for guidance on choosing between backends.

mod error;
mod publisher;
mod subscriber;

pub use error::result_to_transport_error;
pub use publisher::AeronRsPublisher;
pub use subscriber::AeronRsSubscriber;
