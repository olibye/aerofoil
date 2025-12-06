# transport-traits Specification Delta

## ADDED Requirements

### Requirement: Zero-Copy Offer Method
The `AeronPublisher` trait SHALL provide a zero-copy offer method for performance-critical publishing.

#### Scenario: Offer with mutable buffer
- **WHEN** calling `offer_mut` with a mutable message buffer (`&mut [u8]`)
- **THEN** the method publishes without intermediate buffer copies (when backend supports it)

#### Scenario: Choose between ergonomics and performance
- **WHEN** caller has immutable data (e.g., `&[u8]`, `&str`)
- **THEN** use `offer` for convenience (may copy internally)
- **WHEN** caller has mutable buffer and needs maximum performance
- **THEN** use `offer_mut` for zero-copy publishing

#### Scenario: Backend-specific behavior
- **WHEN** using aeron-rs backend with `offer_mut`
- **THEN** buffer is used directly without copying
- **WHEN** using rusteron backend with `offer_mut`
- **THEN** delegates to `offer` (rusteron already accepts `&[u8]`)
