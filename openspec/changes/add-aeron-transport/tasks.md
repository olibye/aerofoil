update # Implementation Tasks

## 1. Core Abstractions
- [ ] 1.1 Define `AeronPublisher` trait with offer/try_claim methods
- [ ] 1.2 Define `AeronSubscriber` trait with poll method
- [ ] 1.3 Create common `TransportError` enum unifying both client error types
- [ ] 1.4 Define buffer types for zero-copy access (publication claim, subscription fragment)

## 2. Feature Flag Configuration
- [ ] 2.1 Add `rusteron` and `aeron-rs` feature flags to Cargo.toml
- [ ] 2.2 Set default feature (select one backend as default)
- [ ] 2.3 Add compile-time checks to ensure mutual exclusivity of backends
- [ ] 2.4 Configure conditional compilation attributes in module structure

## 3. Rusteron Adapter Implementation
- [ ] 3.1 Add rusteron dependency with optional feature flag
- [ ] 3.2 Implement `AeronPublisher` trait for Rusteron publication
- [ ] 3.3 Implement `AeronSubscriber` trait for Rusteron subscription
- [ ] 3.4 Map Rusteron errors to common `TransportError` type
- [ ] 3.5 Ensure zero-copy patterns using Rusteron's buffer access APIs

## 4. Aeron-rs Adapter Implementation
- [ ] 4.1 Add aeron-rs dependency with optional feature flag
- [ ] 4.2 Implement `AeronPublisher` trait for aeron-rs publication
- [ ] 4.3 Implement `AeronSubscriber` trait for aeron-rs subscription
- [ ] 4.4 Map aeron-rs errors to common `TransportError` type
- [ ] 4.5 Ensure zero-copy patterns using aeron-rs buffer access APIs

## 5. Testing
- [ ] 5.1 Write unit tests for trait implementations (mock-based with mockall)
- [ ] 5.2 Add CI matrix to test both feature flag combinations
- [ ] 5.3 Create example demonstrating publication with both backends
- [ ] 5.4 Create example demonstrating subscription with both backends
- [ ] 5.5 Add documentation tests showing usage patterns

## 6. Documentation
- [ ] 6.1 Document trait APIs with rustdoc
- [ ] 6.2 Add crate-level documentation explaining feature flag usage
- [ ] 6.3 Document zero-copy usage patterns
- [ ] 6.4 Add examples to README showing both backend options
