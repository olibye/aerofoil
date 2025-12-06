# transport-traits Specification Delta

## ADDED Requirements

### Requirement: Mutable Buffer Offer Method
The `AeronPublisher` trait SHALL provide an `offer_mut` method accepting `&mut [u8]`.

#### Scenario: Offer with mutable buffer
- **WHEN** calling `offer_mut` with a mutable message buffer (`&mut [u8]`)
- **THEN** the method publishes the message (avoids copy on aeron-rs backend)

#### Scenario: Choose between offer and offer_mut
- **WHEN** caller has immutable data (e.g., `&[u8]`, `&str`)
- **THEN** use `offer` for convenience
- **WHEN** caller has mutable buffer and wants to avoid copies on aeron-rs
- **THEN** use `offer_mut`

#### Scenario: Backend-specific behavior
- **WHEN** using aeron-rs backend with `offer_mut`
- **THEN** buffer is used directly without copying
- **WHEN** using rusteron backend with `offer_mut`
- **THEN** behavior is identical to `offer` (rusteron accepts `&[u8]`)
