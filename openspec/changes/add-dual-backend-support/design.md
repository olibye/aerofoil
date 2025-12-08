# Dual Backend Support Design

## Context
The project supports two Aeron client implementations:
- **rusteron**: C++ bindings via FFI, mature, production-ready
- **aeron-rs**: Pure Rust implementation, no native dependencies

Both share the same media driver (`rusteron-media-driver`) but currently cannot be compiled together due to `#[cfg(all(feature = "aeron-rs", not(feature = "rusteron")))]` guards.

## Goals
- Enable simultaneous compilation of both backends
- Allow benchmarks to compare both backends in a single run
- Generate unified Criterion reports with side-by-side comparisons

## Non-Goals
- Changing the transport trait API
- Runtime backend selection (compile-time selection is preserved)

## Decisions

### Decision: Remove mutual exclusivity constraint
Simply change `#[cfg(all(feature = "aeron-rs", not(feature = "rusteron")))]` to `#[cfg(feature = "aeron-rs")]` throughout the codebase.

**Rationale**: The mutual exclusivity was never technically required - both backends can coexist since they have distinct type names and share the media driver. Users can enable both with `--features rusteron,aeron-rs`.

### Decision: Separate embedded-driver feature
Move `rusteron-media-driver` from `rusteron` and `aeron-rs` feature dependencies to a separate `embedded-driver` feature.

**Rationale**: The media driver is only used in tests and benchmarks (`tests/common/mod.rs`, `benches/common/mod.rs`). The library itself only needs the client to connect to an externally running driver. This separation:
- Reduces dependency footprint for library users
- Makes the distinction clear: client vs embedded driver
- Allows users who run their own media driver (e.g., Java driver) to avoid compiling the C++ driver bindings

### Decision: Unified benchmark groups with backend suffix
Benchmark groups will use naming pattern `{backend}/{operation}` (e.g., `rusteron/offer`, `aeron-rs/offer`) enabling Criterion's comparison features.

**Rationale**: Criterion can compare groups with similar names, making `--baseline` comparisons straightforward.

### Decision: Shared MediaDriverGuard, separate Aeron clients
Both backends will share the `MediaDriverGuard` (already the case) but maintain separate `BenchContext` types in their own modules.

**Rationale**: The media driver is backend-agnostic, but the Aeron client APIs differ significantly between rusteron and aeron-rs.

## Risks / Trade-offs

| Risk | Mitigation |
|------|------------|
| Stream ID conflicts between backends in same run | Use distinct stream ID ranges (rusteron: 1000-1999, aeron-rs: 2000-2999) |
| Benchmark ordering affects results | Run backends in consistent order, document warm-up effects |

## Migration Plan
1. Update cfg guards to remove `not(feature = "rusteron")` conditions
2. Update benchmarks to run both when both features enabled
3. Document new workflow in README

## Open Questions
None - straightforward cfg gate simplification.
