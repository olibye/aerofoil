# Add Zero-Copy Offer Method

## Why
The current `AeronPublisher::offer` trait method takes `&[u8]` (immutable slice), but aeron-rs requires `&mut [u8]` for its `AtomicBuffer::wrap_slice` API. This forces the aeron-rs implementation to copy the buffer before publishing.

Rather than changing the existing API (breaking change), we add a second method for zero-copy publishing when callers have mutable buffers.

## What Changes
- Keep existing `offer(&mut self, buffer: &[u8])` for simple/ergonomic usage
- Add `offer_mut(&mut self, buffer: &mut [u8])` for zero-copy publishing
- Provide default implementation of `offer` that delegates to `offer_mut` (copies if needed)
- aeron-rs: Implements `offer_mut` directly (zero-copy), `offer` copies then calls `offer_mut`
- rusteron: Implements `offer` directly (already zero-copy), `offer_mut` delegates to `offer`

## API Design

```rust
pub trait AeronPublisher {
    /// Offers a message to the publication (may copy internally).
    fn offer(&mut self, buffer: &[u8]) -> Result<i64, TransportError>;

    /// Offers a message without copying (zero-copy when supported).
    fn offer_mut(&mut self, buffer: &mut [u8]) -> Result<i64, TransportError>;
}
```

## Impact
- Affected specs: `transport-traits` (added method)
- Affected code: `src/transport/mod.rs`, both publisher implementations
- Breaking change: No - existing `offer(&[u8])` API unchanged
- Performance: Zero-copy path available via `offer_mut` for aeron-rs
- User value: Choose between ergonomics (`offer`) and performance (`offer_mut`)
