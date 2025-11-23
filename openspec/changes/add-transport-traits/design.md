# Transport Traits Design

## Context
This is the foundational change for the Aerofoil transport abstraction layer. We need trait-based abstractions that allow code to work with different Aeron client implementations (Rusteron, aeron-rs) without runtime overhead. By defining traits first, we establish the API contract before committing to specific backend dependencies.

**Constraints:**
- Zero runtime overhead from abstraction (trait objects not permitted in hot path)
- Support for zero-copy message patterns
- Testability without Aeron infrastructure

**Stakeholders:**
- Future adapter implementations (Rusteron, aeron-rs)
- Application code that will use transports generically
- Test authors who need to mock Aeron behavior

## Goals / Non-Goals

**Goals:**
- Define clean, minimal trait API for publication and subscription
- Enable static dispatch (monomorphization, no vtables)
- Provide mock implementations for testing
- Support zero-copy via lifetime-bound buffer types
- Unified error handling across implementations

**Non-Goals:**
- Implementing real Aeron backends (deferred to later changes)
- Performance benchmarking (needs real implementations)
- Feature flags for backend selection (comes with real adapters)

## Decisions

### Decision 1: Separate Publisher and Subscriber traits
**What:** Define `AeronPublisher` and `AeronSubscriber` as separate traits rather than a single `Transport` trait.

**Why:**
- Most code uses either publishing or subscribing, not both
- Smaller trait surface area is easier to implement and mock
- Allows type-level enforcement (e.g., a processor that only publishes can't accidentally subscribe)
- Follows single responsibility principle

**Alternatives considered:**
- **Combined `AeronTransport` trait:** Would force implementations to provide both even if only one is needed - rejected for flexibility

### Decision 2: Trait methods return Result<T, TransportError>
**What:** All fallible trait methods return a unified `TransportError` type.

**Why:**
- Consistent error handling across implementations
- Can use `?` operator ergonomically
- Error enum can be extended without breaking existing code
- Preserves backend-specific error details via `source()`

**Implementation:**
```rust
pub trait AeronPublisher {
    fn offer(&mut self, buffer: &[u8]) -> Result<i64, TransportError>;
}
```

### Decision 3: Buffer types use lifetime bounds, not ownership
**What:** `ClaimBuffer<'a>` and `FragmentBuffer<'a>` borrow from underlying transport.

**Why:**
- Prevents use-after-free when accessing Aeron-managed buffers
- Zero-copy semantics: buffers are views, not owned data
- Rust lifetime checker ensures safety
- More efficient than copying or reference counting

**Example:**
```rust
pub struct ClaimBuffer<'a> {
    buffer: &'a mut [u8],
    // internal fields...
}
```

### Decision 4: In-memory mock with inspection APIs
**What:** Provide `MockPublisher` and `MockSubscriber` structs with methods to inspect/inject messages.

**Why:**
- Simplest possible testing implementation
- No external dependencies (not even mockall for basic mocks)
- Deterministic behavior for testing
- Can verify messages were published, inject controlled messages

**API:**
```rust
impl MockPublisher {
    pub fn new() -> Self;
    pub fn published_messages(&self) -> &[Vec<u8>];
}

impl MockSubscriber {
    pub fn new() -> Self;
    pub fn inject_message(&mut self, data: Vec<u8>);
}
```

**Plus mockall integration:** Also add `#[automock]` for advanced mocking scenarios.

### Decision 5: Non-blocking semantics enforced by trait contract
**What:** Trait documentation specifies all methods must be non-blocking.

**Why:**
- Critical for HFT use case
- Enforced via documentation contract
- Implementations that block violate trait contract
- Allows caller to use in latency-sensitive contexts safely

**Documentation approach:**
- Document non-blocking guarantee in trait rustdoc
- Each method documents how it handles unavailability (back-pressure, zero poll results)

## Risks / Trade-offs

**Risk:** Trait API might not fit all backend features
- **Impact:** May need trait extensions or breaking changes later
- **Mitigation:** Start minimal (offer, poll, try_claim). Extensions can be added as separate traits or via associated types. Review both Rusteron and aeron-rs APIs before finalizing.

**Trade-off:** Two mock strategies (in-memory + mockall)
- **Benefit:** Flexibility for different testing scenarios
- **Cost:** Two implementations to maintain
- **Justification:** In-memory mocks are simple and deterministic. Mockall handles complex expectations. Both useful.

**Risk:** Lifetime complexity in buffer types
- **Impact:** Users might struggle with lifetime errors
- **Mitigation:** Provide clear documentation and examples. Buffer types are only used in zero-copy paths; simple `offer(&[u8])` method available for ease of use.

## Migration Plan

**This is the first change - no migration needed.**

Future changes build on this foundation:
- Change 2 (`add-rusteron-adapter`) will implement these traits for Rusteron
- Change 3 (`add-aeron-rs-adapter`) will implement these traits for aeron-rs
- Change 4 (`add-transport-benchmarks`) will benchmark implementations

**API Stability:**
Once this change ships, the traits become the stable API contract. Breaking changes should be avoided. Extensions via new traits are preferred.

## Open Questions

1. **Should ClaimBuffer provide length/capacity methods?**
   - Leaning **yes** - useful for writing serialized data
   - Decision before implementation: Review Aeron buffer APIs to ensure we can provide these

2. **Should poll take a closure or return an iterator?**
   - Current design: `poll<F: FnMut(&FragmentBuffer)>(handler: F)`
   - Alternative: Return `impl Iterator<Item = FragmentBuffer>`
   - **Lean toward closure** - matches Aeron's callback model, no allocation

3. **Should we support async traits now or defer?**
   - Not needed for initial HFT use case (sync polling is standard)
   - Can add `AsyncAeronPublisher` trait later if needed
   - **Decision: Defer async support**
