# Rusteron Adapter

## ADDED Requirements

### Requirement: Rusteron Adapter
The library SHALL provide an adapter implementation for the Rusteron Aeron client that implements the transport abstraction traits.

#### Scenario: Rusteron publication
- **WHEN** the Rusteron feature is enabled
- **THEN** messages can be published to Aeron channels using the Rusteron C++ wrapper

#### Scenario: Rusteron subscription
- **WHEN** the Rusteron feature is enabled
- **THEN** messages can be received from Aeron channels using the Rusteron C++ wrapper

#### Scenario: Trait implementation
- **WHEN** using Rusteron types with generic code expecting transport traits
- **THEN** the code compiles and operates correctly

### Requirement: Feature Flag Configuration
The library SHALL support compile-time selection of the Rusteron backend through a Cargo feature flag.

#### Scenario: Feature flag enabled
- **WHEN** the `rusteron` feature is enabled in Cargo.toml
- **THEN** Rusteron dependency is compiled and linked

#### Scenario: Feature flag disabled
- **WHEN** the `rusteron` feature is not enabled
- **THEN** no Rusteron code or dependencies are included in the build

### Requirement: Error Mapping
The library SHALL map Rusteron-specific errors to the common `TransportError` type.

#### Scenario: Rusteron error conversion
- **WHEN** a Rusteron operation fails
- **THEN** the error is converted to `TransportError` with the original error preserved as source

#### Scenario: Back-pressure handling
- **WHEN** Rusteron returns a back-pressure indication
- **THEN** it is mapped to `TransportError::BackPressure` variant

### Requirement: Zero-Copy Message Handling
The library SHALL utilize Rusteron's buffer access APIs to enable zero-copy message patterns.

#### Scenario: Direct buffer access on publication
- **WHEN** claiming a publication buffer
- **THEN** the adapter provides direct access to the Rusteron claim buffer without intermediate copying

#### Scenario: Direct buffer access on subscription
- **WHEN** receiving a message fragment
- **THEN** the adapter provides read-only access to the Rusteron receive buffer without copying

### Requirement: Non-Blocking Operations
The library SHALL ensure Rusteron adapter operations remain non-blocking.

#### Scenario: Publication without blocking
- **WHEN** the Aeron buffer is full
- **THEN** publication attempts return immediately with back-pressure indication

#### Scenario: Subscription polling
- **WHEN** polling for messages
- **THEN** the operation returns immediately whether messages are available or not
