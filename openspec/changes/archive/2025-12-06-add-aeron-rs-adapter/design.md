# Aeron-rs Adapter Design

## Context
This change implements the transport traits for aeron-rs, the pure Rust Aeron client. While `add-rusteron-adapter` provides the mature C++ wrapper, aeron-rs offers a pure Rust alternative for users who prioritize simpler deployment, memory safety guarantees, or targets where C++ toolchain is unavailable (e.g., certain embedded or WASM scenarios).

**Prerequisites:**
- `add-transport-traits` must be completed (traits defined)

**Parallel to:**
- `add-rusteron-adapter` (this is the alternative implementation)

**Constraints:**
- Must not block in publication or subscription operations
- Must preserve zero-copy semantics where aeron-rs supports them
- Pure Rust - no C++ dependencies

**Stakeholders:**
- Users requiring pure Rust deployment
- Teams without C++ toolchain or cross-compilation needs
- Projects prioritizing Rust memory safety throughout the stack

## Goals / Non-Goals

**Goals:**
- Implement `AeronPublisher` and `AeronSubscriber` for aeron-rs types
- Map aeron-rs errors to `TransportError` uniformly
- Enable zero-copy via aeron-rs buffer APIs
- Feature-gate behind `aeron-rs` feature flag
- Ensure mutual exclusivity with `rusteron` feature
- Provide working examples
- Add CI matrix testing both backends

**Non-Goals:**
- Supporting aeron-rs-specific features not in trait contract
- Runtime backend switching
- Performance parity with Rusteron (nice-to-have, but aeron-rs may be slower)

## Decisions

### Decision 1: Mutual exclusivity via compile_error!
**What:** Add compile-time check preventing both `rusteron` and `aeron-rs` features from being enabled.

**Why:**
- Prevents ambiguous backend selection
- Clear error message at compile time
- No runtime overhead or complexity

**Implementation:**
```rust
#[cfg(all(feature = "rusteron", feature = "aeron-rs"))]
compile_error!("Cannot enable both 'rusteron' and 'aeron-rs' features. Choose one backend.");
```

### Decision 2: aeron-rs is opt-in, Rusteron is default
**What:** Rusteron remains default feature. Users explicitly opt into aeron-rs with `default-features = false, features = ["aeron-rs"]`.

**Why:**
- Rusteron is more mature and proven
- HFT use case favors battle-tested implementation
- Pure Rust users explicitly choose their preference
- Matches decision from `add-rusteron-adapter`

### Decision 3: Parallel module structure to Rusteron
**What:** `src/transport/aeron_rs/` mirrors the structure of `src/transport/rusteron/`.

**Why:**
- Consistency makes code easier to navigate
- Similar implementation patterns (both wrap underlying client types)
- Easier to maintain and review

**Structure:**
```
src/transport/
├── aeron_rs/
│   ├── mod.rs
│   ├── publisher.rs
│   ├── subscriber.rs
│   └── error.rs
└── rusteron/
    ├── mod.rs
    ├── publisher.rs
    ├── subscriber.rs
    └── error.rs
```

### Decision 4: Wrapper structs own aeron-rs types
**What:** `AeronRsPublisher` and `AeronRsSubscriber` own the underlying aeron-rs types.

**Why:**
- Same rationale as Rusteron adapter
- Clear ownership semantics
- Allows additional state if needed

**Implementation:**
```rust
pub struct AeronRsPublisher {
    publication: aeron::Publication,
}

impl AeronPublisher for AeronRsPublisher {
    fn offer(&mut self, buffer: &[u8]) -> Result<i64, TransportError> {
        self.publication.offer(buffer)
            .map_err(TransportError::from)
    }
}
```

### Decision 5: CI matrix tests both backends
**What:** Add GitHub Actions (or similar) jobs testing with `--features rusteron` and `--features aeron-rs`.

**Why:**
- Ensures both backends work correctly
- Catches backend-specific regressions
- Validates mutual exclusivity check
- Standard practice for optional features

**Jobs:**
- Test with Rusteron (default features)
- Test with aeron-rs (explicit feature)
- Test with no default features + mocks only

## Risks / Trade-offs

**Risk:** aeron-rs is less mature than Rusteron
- **Impact:** May have bugs, missing features, or performance issues
- **Mitigation:**
  - Document maturity difference clearly
  - Make Rusteron the default
  - Encourage user feedback and bug reports to aeron-rs project

**Trade-off:** Pure Rust vs Performance
- **Benefit:** Simpler deployment, memory safety, no C++ toolchain
- **Cost:** Potentially slower than C++ Aeron client wrapped by Rusteron
- **Justification:** Users can choose based on their priorities. Benchmarks (from `add-transport-benchmarks`) will quantify difference.

**Risk:** aeron-rs API might differ from Rusteron
- **Impact:** Implementation patterns might diverge, making maintenance harder
- **Mitigation:** Trait contract enforces API uniformity. Implementation details can differ as long as contract is met.

**Risk:** aeron-rs might not support all features
- **Impact:** Some advanced Aeron features might not be available
- **Mitigation:** Start with minimal viable feature set (offer, poll, try_claim). Document limitations. Users needing advanced features can use Rusteron.

## Migration Plan

**Prerequisites:**
1. `add-transport-traits` must be deployed
2. Ideally `add-rusteron-adapter` is deployed (validates trait design)

**Deployment:**
1. Release this change as optional feature
2. Document feature flag selection clearly
3. Users continue using Rusteron by default
4. Users can opt into aeron-rs for pure Rust deployment

**Future:**
- `add-transport-benchmarks` will compare performance objectively
- Users can make informed backend choice based on data

**Rollback:**
Simple revert if critical issues found - aeron-rs is opt-in, so no existing users affected.

## Open Questions

1. **Does aeron-rs support buffer claiming for zero-copy publication?**
   - Need to investigate aeron-rs API documentation
   - If not available, `try_claim` might need to use a different approach or return error
   - **Action: Review aeron-rs docs before implementation**

2. **How does aeron-rs handle fragment assembly?**
   - Similar to Rusteron concern
   - Need to test with large messages
   - **Action: Integration test with messages exceeding MTU**

3. **Should we add a feature matrix table to documentation?**
   - Could document which features work with which backend
   - Example: "Zero-copy publication: ✓ Rusteron, ? aeron-rs (pending investigation)"
   - **Lean toward: Yes, add to crate-level docs once both adapters implemented**

4. **Should we support running both backends in same binary for benchmarking?**
   - Current design: mutual exclusivity
   - Alternative: Allow both for benchmarking purposes only
   - **Decision: Keep mutual exclusivity. Separate builds for benchmarks is acceptable.**
