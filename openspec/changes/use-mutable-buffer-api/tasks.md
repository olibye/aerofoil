# Implementation Tasks

## 1. Update Trait Definition
- [x] 1.1 Add `offer_mut(&mut self, buffer: &mut [u8])` method to `AeronPublisher` trait
- [x] 1.2 Document both methods explaining when to use each

## 2. Update Rusteron Implementation
- [x] 2.1 Add `offer_mut` that delegates to existing `offer` (rusteron accepts both)

## 3. Update Aeron-rs Implementation
- [x] 3.1 Rename current `offer` logic to `offer_mut` (zero-copy path)
- [x] 3.2 Implement `offer` to copy buffer then call `offer_mut`
- [x] 3.3 Remove the workaround comment about cloning

## 4. Update Spec
- [x] 4.1 Add scenario for `offer_mut` zero-copy method to `transport-traits` spec

## 5. Validation
- [x] 5.1 Run `cargo build --features rusteron`
- [x] 5.2 Run `cargo build --no-default-features --features aeron-rs`
- [x] 5.3 Run `cargo test --features rusteron --lib`
- [x] 5.4 Run `cargo test --no-default-features --features aeron-rs --lib`
- [x] 5.5 Run `cargo clippy --features rusteron`
- [x] 5.6 Run `cargo clippy --no-default-features --features aeron-rs`
