//! Aerofoil: Wingfoil Aeron adapters for high-frequency trading
//!
//! This library provides transport abstractions and adapters for integrating
//! Aeron messaging with Wingfoil stream processing, enabling low-latency,
//! stateful message processing for HFT systems.
//!
//! # Features
//!
//! - **`rusteron`** (default): Enables Rusteron adapter (requires C++ toolchain)
//! - **`aeron-rs`**: Enables pure Rust aeron-rs adapter (no C++ toolchain required)
//!
//! Note: `rusteron` and `aeron-rs` features are mutually exclusive.
//!
//! # Architecture
//!
//! The library is organized into:
//!
//! - [`transport`]: Trait-based transport abstractions and Aeron client adapters
//! - [`nodes`]: Wingfoil node implementations for Aeron integration

// Compile-time check: rusteron and aeron-rs features are mutually exclusive
#[cfg(all(feature = "rusteron", feature = "aeron-rs"))]
compile_error!("Cannot enable both 'rusteron' and 'aeron-rs' features. Choose one backend.");

pub mod nodes;
pub mod transport;
