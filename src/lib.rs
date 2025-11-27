//! Aerofoil: Wingfoil Aeron adapters for high-frequency trading
//!
//! This library provides transport abstractions and adapters for integrating
//! Aeron messaging with Wingfoil stream processing, enabling low-latency,
//! stateful message processing for HFT systems.
//!
//! # Features
//!
//! - **`rusteron`** (default): Enables Rusteron adapter (requires C++ toolchain)
//!
//! # Architecture
//!
//! The library is organized into:
//!
//! - [`transport`]: Trait-based transport abstractions and Aeron client adapters

pub mod transport;
