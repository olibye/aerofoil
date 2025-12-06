# aeron-rs-adapter Specification

## Purpose
TBD - created by archiving change add-aeron-rs-adapter. Update Purpose after archive.
## Requirements
### Requirement: Aeron-rs Adapter
The library SHALL provide an adapter implementation for the pure Rust aeron-rs client that implements the transport abstraction traits.

#### Scenario: Aeron-rs publication
- **WHEN** the aeron-rs feature is enabled
- **THEN** messages can be published to Aeron channels using the pure Rust client

#### Scenario: Aeron-rs subscription
- **WHEN** the aeron-rs feature is enabled
- **THEN** messages can be received from Aeron channels using the pure Rust client

#### Scenario: Trait implementation
- **WHEN** using aeron-rs types with generic code expecting transport traits
- **THEN** the code compiles and operates correctly

### Requirement: Feature Flag Configuration
The library SHALL support compile-time selection of the aeron-rs backend through a Cargo feature flag with mutual exclusivity from Rusteron.

#### Scenario: Feature flag enabled
- **WHEN** the `aeron-rs` feature is enabled in Cargo.toml
- **THEN** aeron-rs dependency is compiled and linked

#### Scenario: Feature flag disabled
- **WHEN** the `aeron-rs` feature is not enabled
- **THEN** no aeron-rs code or dependencies are included in the build

#### Scenario: Mutual exclusivity check
- **WHEN** both `rusteron` and `aeron-rs` features are enabled simultaneously
- **THEN** a compile-time error is raised preventing ambiguous backend selection

### Requirement: Error Mapping
The library SHALL map aeron-rs-specific errors to the common `TransportError` type.

#### Scenario: Aeron-rs error conversion
- **WHEN** an aeron-rs operation fails
- **THEN** the error is converted to `TransportError` with the original error preserved as source

#### Scenario: Back-pressure handling
- **WHEN** aeron-rs returns a back-pressure indication
- **THEN** it is mapped to `TransportError::BackPressure` variant

### Requirement: Zero-Copy Message Handling
The library SHALL utilize aeron-rs buffer access APIs to enable zero-copy message patterns.

#### Scenario: Direct buffer access on publication
- **WHEN** claiming a publication buffer
- **THEN** the adapter provides direct access to the aeron-rs claim buffer without intermediate copying

#### Scenario: Direct buffer access on subscription
- **WHEN** receiving a message fragment
- **THEN** the adapter provides read-only access to the aeron-rs receive buffer without copying

### Requirement: Non-Blocking Operations
The library SHALL ensure aeron-rs adapter operations remain non-blocking.

#### Scenario: Publication without blocking
- **WHEN** the Aeron buffer is full
- **THEN** publication attempts return immediately with back-pressure indication

#### Scenario: Subscription polling
- **WHEN** polling for messages
- **THEN** the operation returns immediately whether messages are available or not

