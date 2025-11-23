# Aeron Transport Adapters Design

## Context
The aerofoil library needs to provide Wingfoil adapters for Aeron messaging, but there are two viable Rust Aeron client options:
1. **Rusteron**: Wrapper around the official C++ Aeron client - mature, battle-tested, but requires C++ toolchain
2. **aeron-rs**: Pure Rust implementation - simpler deployment, memory safe, but less mature

Different deployment scenarios favor different backends. High-frequency trading systems demand flexibility without sacrificing performance. This design creates zero-cost abstractions that allow compile-time backend selection.

**Constraints:**
- Non-blocking, low-latency code paths (critical for HFT)
- Zero-copy message handling where possible
- No runtime overhead from abstractions

**Stakeholders:**
- Library users who need deployment flexibility
- Systems requiring pure Rust (e.g., WASM targets, simplified builds)
- Performance-critical applications that cannot tolerate abstraction overhead

## Goals / Non-Goals

**Goals:**
- Compile-time polymorphism via traits (zero runtime cost)
- Support both Rusteron and aeron-rs with identical application code
- Maintain zero-copy patterns where underlying clients support them
- Unified error handling across backends
- Simple feature flag selection

**Non-Goals:**
- Runtime backend switching (adds overhead, violates zero-cost principle)
- Supporting additional Aeron clients beyond these two initially
- Abstracting over non-Aeron transports
- Implementing Aeron client features not exposed by both libraries

## Decisions

### Decision 1: Trait-based abstraction with static dispatch
**What:** Define `AeronPublisher` and `AeronSubscriber` traits implemented by wrapper types around each client.

**Why:**
- Rust traits with static dispatch provide zero-cost abstraction
- Monomorphization eliminates vtable overhead
- Type system enforces API consistency across backends
- Enables compile-time optimization specific to each backend

**Alternatives considered:**
- **Enum wrapper with match statements:** Adds branching overhead in hot path - rejected for performance
- **Macro-based code generation:** Harder to maintain, worse IDE support - rejected for ergonomics
- **Dynamic dispatch with trait objects:** vtable overhead unacceptable for HFT - rejected

### Decision 2: Mutually exclusive Cargo feature flags
**What:** Use `rusteron` and `aeron-rs` feature flags, with compile-time error if both enabled.

**Why:**
- Standard Rust pattern for optional dependencies
- Cargo handles conditional compilation automatically
- Clear, explicit selection at build time
- No binary bloat from unused backend

**Implementation:**
```rust
#[cfg(all(feature = "rusteron", feature = "aeron-rs"))]
compile_error!("Cannot enable both 'rusteron' and 'aeron-rs' features");
```

### Decision 3: Default to Rusteron
**What:** Make `rusteron` the default feature if neither is explicitly selected.

**Why:**
- Rusteron wraps the official C++ client - more mature and battle-tested
- Matches production HFT requirements where performance is critical
- Users wanting pure Rust can explicitly opt into aeron-rs

**Trade-off:** Requires C++ toolchain by default, but aligns with project's HFT focus.

### Decision 4: Buffer abstraction with lifetime-bound references
**What:** Provide buffer types that wrap underlying client buffers with Rust lifetime guarantees.

**Why:**
- Prevents use-after-free when accessing Aeron buffers
- Zero-copy is only safe if lifetimes prevent dangling pointers
- Both clients provide buffer access - abstraction maintains safety

**Example:**
```rust
pub struct PublicationClaim<'a> {
    buffer: &'a mut [u8],
    // backend-specific fields...
}
```

### Decision 5: Unified error type with source preservation
**What:** Create `TransportError` enum covering all error cases from both clients, preserving original error as source.

**Why:**
- Application code can handle errors uniformly
- `std::error::Error::source()` preserves backend-specific debugging info
- Idiomatic Rust error handling with `Result<T, TransportError>`

## Risks / Trade-offs

**Risk:** API divergence between Rusteron and aeron-rs
- **Impact:** Features available in one but not the other cannot be exposed in common trait
- **Mitigation:** Start with minimal viable API covering common operations (publish, subscribe, poll). Extended features can use backend-specific extension traits.

**Risk:** Zero-copy patterns differ between implementations
- **Impact:** One backend might require copying where the other doesn't
- **Mitigation:** Trait contract specifies *intent* for zero-copy; implementation does best effort. Document which operations guarantee zero-copy per backend.

**Trade-off:** Static selection prevents runtime switching
- **Benefit:** Zero runtime overhead, fully optimized binaries
- **Cost:** Need separate builds to compare backends
- **Justification:** HFT context prioritizes performance over flexibility; users needing comparison can script separate builds

**Risk:** Pure Rust client (aeron-rs) is less mature
- **Impact:** May have bugs or missing features
- **Mitigation:** Default to Rusteron; aeron-rs is opt-in. Document maturity difference.

## Migration Plan

**Phase 1 - Foundation (this change):**
1. Implement core traits and both adapters
2. Add basic tests with mocks
3. Provide simple examples

**Phase 2 - Validation (future):**
1. Integration testing against real Aeron media driver
2. Performance benchmarking to verify zero-cost abstraction
3. Example integration with Wingfoil processors

**Phase 3 - Production Readiness (future):**
1. CPU pinning support for latency-sensitive deployments
2. Advanced Aeron features (fragmentation, flow control)
3. Comprehensive error recovery patterns

**Rollback:**
Since this is foundational work with no existing users, rollback is simply reverting the commit. No migration needed.

## Open Questions

1. **Should we support dynamic feature detection at compile time?**
   - Could use `#[cfg(any(...))]` to allow compilation without *any* backend for testing
   - Leaning **no** - every build should select a real backend

2. **How to handle Aeron media driver configuration?**
   - Both clients need driver context/configuration
   - Options: (a) Separate config per backend, (b) Unified config that translates
   - **Decision needed before implementation:** Lean toward (a) for simplicity

3. **Should examples be separate binaries or integration tests?**
   - Separate binaries easier to run manually
   - Integration tests better for CI
   - **Lean toward:** Both - examples/ dir for manual use, tests/ for CI
