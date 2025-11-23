# Rusteron Adapter Design

## Context
This change implements the transport traits for Rusteron, the wrapper around the official C++ Aeron client. Rusteron provides access to the mature, battle-tested Aeron implementation used in production HFT systems. This is the first concrete implementation of the traits defined in `add-transport-traits`.

**Prerequisites:**
- `add-transport-traits` must be completed (traits defined)

**Constraints:**
- Must not block in publication or subscription operations
- Must preserve zero-copy semantics where Rusteron supports them
- Requires C++ toolchain for compilation

**Stakeholders:**
- Production HFT systems requiring proven Aeron performance
- Users who prioritize maturity over pure Rust deployment

## Goals / Non-Goals

**Goals:**
- Implement `AeronPublisher` and `AeronSubscriber` for Rusteron types
- Map Rusteron errors to `TransportError` uniformly
- Enable zero-copy via Rusteron's buffer APIs
- Feature-gate behind `rusteron` feature flag
- Provide working examples

**Non-Goals:**
- Supporting Rusteron-specific features not in trait contract
- Runtime backend switching
- Pure Rust deployment (requires C++ toolchain)

## Decisions

### Decision 1: Rusteron as default feature
**What:** Make `rusteron` the default feature in Cargo.toml.

**Why:**
- Rusteron wraps official C++ client - most mature option
- Matches production HFT requirements
- Users can explicitly opt out with `default-features = false`

**Trade-off:** Requires C++ toolchain by default, but this aligns with project's HFT focus.

### Decision 2: Wrapper structs own Rusteron types
**What:** `RusteronPublisher` and `RusteronSubscriber` own the underlying Rusteron publication/subscription objects.

**Why:**
- Clear ownership semantics
- Rusteron types are not Send/Sync, so wrapping them allows controlled access
- Enables additional state (e.g., cached configuration) if needed

**Implementation:**
```rust
pub struct RusteronPublisher {
    publication: rusteron::Publication,
}

impl AeronPublisher for RusteronPublisher {
    fn offer(&mut self, buffer: &[u8]) -> Result<i64, TransportError> {
        self.publication.offer(buffer)
            .map_err(TransportError::from)
    }
}
```

### Decision 3: Map Rusteron Result types to TransportError
**What:** Implement `From<rusteron::Error>` for `TransportError`.

**Why:**
- Idiomatic Rust error handling
- Enables `?` operator
- Preserves original error via `source()`

**Implementation:**
```rust
impl From<rusteron::AeronError> for TransportError {
    fn from(err: rusteron::AeronError) -> Self {
        match err {
            rusteron::AeronError::BackPressure => TransportError::BackPressure,
            // other mappings...
            _ => TransportError::Backend(Box::new(err)),
        }
    }
}
```

### Decision 4: Use Rusteron's tryClaim for zero-copy publication
**What:** Implement `try_claim` using Rusteron's buffer claim API.

**Why:**
- Rusteron exposes `tryClaim` which returns a mutable buffer view
- Avoids intermediate copy - write directly to Aeron buffer
- Matches Aeron's intended usage pattern for low-latency

**Lifetime management:** Claim returns a guard object. Wrap it in our `ClaimBuffer<'a>` type with appropriate lifetime bounds.

### Decision 5: Fragment handler callback for subscription
**What:** Use Rusteron's polling API with fragment handler callback.

**Why:**
- Rusteron subscription works via callbacks - natural fit for our `poll<F>` design
- No allocation needed for iteration
- Matches how Aeron subscriptions work natively

**Implementation:**
```rust
impl AeronSubscriber for RusteronSubscriber {
    fn poll<F>(&mut self, mut handler: F) -> Result<usize, TransportError>
    where
        F: FnMut(&FragmentBuffer) -> Result<(), TransportError>,
    {
        let count = self.subscription.poll(|buffer, header| {
            let fragment = FragmentBuffer::from_rusteron(buffer);
            handler(&fragment).unwrap(); // TODO: handle errors properly
        })?;
        Ok(count)
    }
}
```

## Risks / Trade-offs

**Risk:** Rusteron API might change
- **Impact:** Breaking changes in Rusteron could break our adapter
- **Mitigation:** Pin Rusteron version in Cargo.toml. Monitor Rusteron releases.

**Trade-off:** C++ toolchain requirement
- **Benefit:** Access to mature, official Aeron client
- **Cost:** Requires C++ compiler, complicates cross-compilation
- **Justification:** HFT use case prioritizes performance and maturity. Pure Rust option available via `add-aeron-rs-adapter`.

**Risk:** Rusteron types not Send/Sync
- **Impact:** Cannot move publisher/subscriber across threads easily
- **Mitigation:** Document thread affinity requirements. HFT systems typically pin to specific cores anyway.

**Risk:** Error mapping might lose information
- **Impact:** Some Rusteron-specific error details might not fit `TransportError` enum
- **Mitigation:** Use `TransportError::Backend(Box<dyn Error>)` variant to wrap unmappable errors. Preserve original via `source()`.

## Migration Plan

**Prerequisites:**
1. `add-transport-traits` must be deployed first
2. Ensure users have C++ toolchain available

**Deployment:**
1. Release this change with `rusteron` as default feature
2. Users get working Aeron support out of the box
3. Examples demonstrate usage

**Future:**
- `add-aeron-rs-adapter` provides alternative pure Rust backend
- Users can choose via feature flags

**Rollback:**
Simple revert if issues found - no users yet dependent on this.

## Open Questions

1. **Should we expose Rusteron configuration options?**
   - Rusteron has many configuration knobs (channel, stream ID, etc.)
   - Options: (a) Builder pattern for configuration, (b) Pass through Rusteron context
   - **Decision before implementation:** Start with simple constructor taking channel string. Add builder later if needed.

2. **How to handle fragment assembly?**
   - Aeron can fragment large messages
   - Rusteron might handle this automatically, or we may need to
   - **Need to investigate:** Check Rusteron documentation and test with large messages

3. **Should we support controlled/exclusive publications?**
   - Aeron has different publication types
   - Trait design is generic - could support via separate constructor methods
   - **Decision: Start with regular publication, add variants if needed**
