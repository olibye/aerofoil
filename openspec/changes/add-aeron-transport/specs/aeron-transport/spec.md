# Aeron Transport Adapters

## ADDED Requirements

### Requirement: Transport Abstraction
The library SHALL provide trait-based abstractions for Aeron transport operations that enable zero-cost, compile-time polymorphism across different Aeron client implementations.

#### Scenario: Publication abstraction
- **WHEN** application code publishes a message using the transport trait
- **THEN** the concrete implementation (Rusteron or aeron-rs) is selected at compile time with zero runtime overhead

#### Scenario: Subscription abstraction
- **WHEN** application code subscribes to a channel using the transport trait
- **THEN** the concrete implementation handles message reception according to the selected backend

### Requirement: Rusteron Adapter
The library SHALL provide an adapter implementation for the Rusteron Aeron client that implements the transport abstraction traits.

#### Scenario: Rusteron publication
- **WHEN** the Rusteron feature is enabled
- **THEN** messages can be published to Aeron channels using the Rusteron C++ wrapper

#### Scenario: Rusteron subscription
- **WHEN** the Rusteron feature is enabled
- **THEN** messages can be received from Aeron channels using the Rusteron C++ wrapper

### Requirement: Aeron-rs Adapter
The library SHALL provide an adapter implementation for the pure Rust aeron-rs client that implements the transport abstraction traits.

#### Scenario: Aeron-rs publication
- **WHEN** the aeron-rs feature is enabled
- **THEN** messages can be published to Aeron channels using the pure Rust client

#### Scenario: Aeron-rs subscription
- **WHEN** the aeron-rs feature is enabled
- **THEN** messages can be received from Aeron channels using the pure Rust client

### Requirement: Feature Flag Selection
The library SHALL support compile-time transport selection through Cargo feature flags with mutually exclusive backend options.

#### Scenario: Single backend selection
- **WHEN** exactly one transport feature is enabled
- **THEN** only that transport's dependencies are compiled and linked

#### Scenario: Default backend
- **WHEN** no transport feature is explicitly enabled
- **THEN** a sensible default transport is selected (to be determined during implementation)

#### Scenario: Multiple backend conflict
- **WHEN** multiple transport features are enabled simultaneously
- **THEN** a compile-time error SHALL be raised to prevent ambiguity

### Requirement: Zero-Copy Message Handling
The library SHALL support zero-copy message patterns where the underlying Aeron client permits, avoiding unnecessary memory allocations in the critical path.

#### Scenario: Direct buffer access on publication
- **WHEN** publishing a message
- **THEN** the adapter SHALL provide direct access to the Aeron claim buffer when possible

#### Scenario: Direct buffer access on subscription
- **WHEN** receiving a message
- **THEN** the adapter SHALL provide read-only access to the Aeron receive buffer without copying

### Requirement: Unified Error Handling
The library SHALL provide a common error type that unifies error conditions across both Rusteron and aeron-rs implementations.

#### Scenario: Transport error mapping
- **WHEN** an underlying transport client returns an error
- **THEN** it is mapped to the common error type with sufficient detail for debugging

#### Scenario: Error propagation
- **WHEN** a transport operation fails
- **THEN** the error can be handled uniformly regardless of the selected backend

### Requirement: Non-Blocking Operations
The library SHALL ensure all transport operations remain non-blocking to meet low-latency requirements for high-frequency trading systems.

#### Scenario: Publication without blocking
- **WHEN** the Aeron buffer is full
- **THEN** publication attempts SHALL return immediately with a back-pressure indication rather than blocking

#### Scenario: Subscription polling
- **WHEN** polling for messages
- **THEN** the operation SHALL return immediately whether messages are available or not
