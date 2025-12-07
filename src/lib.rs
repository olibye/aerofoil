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
//! Note: `rusteron` and `aeron-rs` features can be enabled simultaneously.
//!
//! # Choosing a Backend
//!
//! | Aspect | rusteron | aeron-rs |
//! |--------|----------|----------|
//! | Implementation | C++ wrapper (FFI) | Pure Rust |
//! | Maturity | More mature | Less mature |
//! | C++ toolchain | Required | Not required |
//! | Cross-compilation | Complex | Simpler |
//! | Performance | Production-tested | See benchmarks |
//!
//! Use `rusteron` (default) for production deployments with established toolchains.
//! Use `aeron-rs` for pure Rust builds or simpler cross-compilation.
//!
//! # Publishing Methods
//!
//! The [`AeronPublisher`](transport::AeronPublisher) trait provides two offer methods:
//!
//! - `offer(&[u8])`: Accepts immutable buffer (may copy internally on aeron-rs)
//! - `offer_mut(&mut [u8])`: Accepts mutable buffer (no copy on aeron-rs)
//!
//! Use `offer_mut` when you have a mutable buffer and want to avoid copies on aeron-rs.
//!
//! # Architecture
//!
//! The library is organized into:
//!
//! - [`transport`]: Trait-based transport abstractions and Aeron client adapters
//! - [`nodes`]: Wingfoil node implementations for Aeron integration


pub mod nodes;
pub mod transport;
