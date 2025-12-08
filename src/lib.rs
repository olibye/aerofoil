//! Aerofoil: Wingfoil Aeron adapters for high-frequency trading
//!
//! This library provides transport abstractions and adapters for integrating
//! Aeron messaging with Wingfoil stream processing, enabling low-latency,
//! stateful message processing for HFT systems.
//!
//! Uses [Rusteron](https://crates.io/crates/rusteron-client) (C++ bindings via FFI) for Aeron connectivity.
//!
//! # Features
//!
//! - **`embedded-driver`**: Enables embedded media driver for tests/benchmarks
//! - **`external-driver`**: Use an external media driver (Java or C++ aeronmd)
//!
//! # Publishing Methods
//!
//! The [`AeronPublisher`](transport::AeronPublisher) trait provides:
//!
//! - `offer(&[u8])`: Publish a message
//! - `try_claim(len)`: Claim buffer for zero-copy writing
//!
//! # Architecture
//!
//! The library is organized into:
//!
//! - [`transport`]: Trait-based transport abstractions and Aeron client adapters
//! - [`nodes`]: Wingfoil node implementations for Aeron integration

pub mod nodes;
pub mod transport;
