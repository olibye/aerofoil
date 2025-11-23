//! Rusteron adapter for Aeron transport traits.
//!
//! This module provides implementations of [`AeronPublisher`](crate::transport::AeronPublisher)
//! and [`AeronSubscriber`](crate::transport::AeronSubscriber) using the Rusteron client,
//! which wraps the official C++ Aeron client.
//!
//! # Features
//!
//! - **Mature implementation**: Uses the battle-tested C++ Aeron client
//! - **Zero-copy support**: Leverages Rusteron's `try_claim` API for zero-copy publication
//! - **Non-blocking**: All operations return immediately
//! - **Production-ready**: Proven performance in high-frequency trading systems
//!
//! # Requirements
//!
//! - C++ toolchain (Clang, CMake) for compilation
//! - Running Aeron Media Driver
//! - Feature flag: `rusteron` (enabled by default)
//!
//! # Example
//!
//! ```ignore
//! use aerofoil::transport::AeronPublisher;
//! use aerofoil::transport::rusteron::RusteronPublisher;
//!
//! // Create publisher (requires media driver running)
//! let publisher = RusteronPublisher::new("aeron:ipc", 1001)?;
//!
//! // Publish message
//! publisher.offer(b"Hello, Aeron!")?;
//! ```

pub mod error;
pub mod publisher;
pub mod subscriber;

#[cfg(test)]
pub mod media_driver;

pub use error::*;
pub use publisher::RusteronPublisher;
pub use subscriber::RusteronSubscriber;
