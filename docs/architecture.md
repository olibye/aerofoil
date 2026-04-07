# Architecture: Aerofoil

## Purpose

Aerofoil is a Rust library providing Wingfoil adapters for Aeron messaging. It enables zero-copy, low-latency message processing through trait-based transport abstractions and peek-based node composition.

## Technology Stack

| Component | Tech | Role |
| :--- | :--- | :--- |
| **Core Framework** | [Wingfoil](https://docs.rs/wingfoil/latest/wingfoil/) | Stream processing graph framework |
| **Transport (default)** | [Rusteron](https://github.com/gsrxyz/rusteron) | C++ Aeron client FFI wrapper |
| **Transport (pure Rust)** | [aeron-rs](https://github.com/UnitedTraders/aeron-rs) | Pure Rust Aeron client |
| **Mocking** | [Mockall](https://github.com/asomers/mockall) | Unit test mock generation |
| **Benchmarking** | [Criterion](https://docs.rs/criterion/latest/criterion/) | Statistical benchmarking |

## Domain Context

- High frequency trading systems
- Stateful stream processing for position keeping
- Non-blocking, low-latency input/output code paths

## Architecture Patterns

### Transport Trait Abstraction

Trait-based polymorphism enables zero-cost switching between rusteron and aeron-rs backends:

- **`AeronPublisher`** trait: `offer(&[u8])`, `offer_mut(&mut [u8])`, `try_claim()` methods
- **`AeronSubscriber`** trait: `poll()` for non-blocking message reception
- **`TransportError`** unified error type with back-pressure variant
- **Zero-copy buffers**: `ClaimBuffer` and `FragmentBuffer` with lifetime guarantees
- **Encapsulation**: No `inner()` methods exposing underlying implementations

### Dual Backend Support

Both rusteron and aeron-rs compile simultaneously for side-by-side comparison:

| Feature Flag | Purpose |
| :--- | :--- |
| `embedded-driver` | Rusteron media driver (for CI/testing) |
| `external-driver` | aeron-rs with external media driver |
| `dhat-heap` | Allocation tracking for benchmarks |

Rusteron-client is a regular dependency (not feature-gated). The `offer_mut(&mut [u8])` method enables zero-copy publishing on aeron-rs while delegating to `offer` on rusteron.

### Peek-Based Node Composition

Wingfoil nodes use peek-based composition for reading upstream state without ownership transfer:

**Reference-based access** (`StreamPeekRef<T>`):
- `AeronSubscriberValueRefNode<T, F, S>` for types with complex lifetimes
- Downstream nodes accept upstream via `Rc<RefCell<T>>`
- Dual-Rc pattern for graph integration

**Value-based access** (`StreamPeek<T>`):
- `AeronSubscriberValueNode<T, F, S>` for cheap-to-clone types
- Simpler `peek_value()` API
- Shared implementation via private `AeronSubscriberCore<T, F, S>`

**Change detection**: Compare peeked values between cycles.

### Subscriber Node Builder

`AeronSubscriberNodeBuilder` provides a fluent API eliminating `Rc<RefCell<>>` boilerplate:

- `.subscriber()`, `.parser()`, `.default()` method chain
- `build()` produces value node, `build_ref()` produces reference node
- Type safety enforced at compile time

### Fan-Out Pattern

Single subscriber feeding multiple downstream processing nodes, each publishing results to separate Aeron streams via callbacks.

## Wingfoil Node Conventions

- Nodes are single-threaded, execute within Wingfoil's graph context
- Node state does NOT use thread-safe primitives (`Arc<Mutex<>>`, `Arc<RwLock<>>`) for internal processing
- Nodes communicate via Wingfoil streams and channels, not shared memory
- `Arc<Mutex<>>` acceptable ONLY for:
  - Test verification where external code observes node state after graph execution
  - Interfacing with external multi-threaded systems outside the graph
- Production nodes expose state via Wingfoil's Stream trait
- A Node's `cycle` function should only output true if there's a new value
- Use static dispatch in hot paths; ban dynamic traits in hot paths

## Design Decisions

### ADR: Separate Node Types Over Combined

Two separate node types (`ValueNode` and `ValueRefNode`) instead of one implementing both traits. Rationale: graph integration constraints and performance - each type has a single responsibility.

### ADR: Static Dispatch in Hot Paths

No `dyn Trait` in hot paths. Static dispatch via generics ensures zero-cost abstractions for transport operations.

### ADR: Binary Message Parsing

Little-endian i64 binary parsing with zero-copy from Rusteron fragment buffers. Stateful nodes (e.g., `SummingNode`) maintain running aggregates across cycles.

### ADR: Feature Flags Over Runtime Selection

Backend selection is compile-time via feature flags, not runtime. This eliminates branching overhead in the hot path and enables dead code elimination.

## Key Constraints

- Input and output code paths must be non-blocking and low latency
- Zero-copy message handling where possible
- Configuration object model abstracts the configuration source
- Consider processor and OS specific optimisation for high performance
- Support processor pinning to specific CPU cores
- Aeron for all signals including logging and monitoring in production, with fallback to stdout/stderr in development

## External Dependencies

### Aeron Media Driver

- Required for both rusteron and aeron-rs transport backends
- Build: `git clone https://github.com/real-logic/aeron && cd aeron && ./cppbuild/cppbuild --build-aeron-driver --no-tests`
- Must be running before starting applications using Aeron transports
- See [Development Guide - Aerofoil](../aerofoil/docs/development-guide.md) for installation

### C++ Toolchain (Rusteron Only)

- Required to compile rusteron crate and its C++ Aeron bindings
- Components: C++17 compatible compiler, CMake, standard build tools
