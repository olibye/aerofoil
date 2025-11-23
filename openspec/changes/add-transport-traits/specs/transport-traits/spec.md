# Transport Traits

## ADDED Requirements

### Requirement: Transport Abstraction
The library SHALL provide trait-based abstractions for Aeron transport operations that enable zero-cost, compile-time polymorphism across different Aeron client implementations.

#### Scenario: Publication abstraction
- **WHEN** application code publishes a message using the transport trait
- **THEN** the concrete implementation is selected at compile time with zero runtime overhead

#### Scenario: Subscription abstraction
- **WHEN** application code subscribes to a channel using the transport trait
- **THEN** the concrete implementation handles message reception according to the selected backend

#### Scenario: Generic code using traits
- **WHEN** writing code generic over `AeronPublisher` or `AeronSubscriber`
- **THEN** the code compiles and works with any implementation (mock or real backend)

### Requirement: Publisher Trait
The library SHALL define an `AeronPublisher` trait with methods for publishing messages to Aeron channels.

#### Scenario: Offer message
- **WHEN** calling the offer method with a message buffer
- **THEN** the method attempts to publish and returns success or back-pressure indication

#### Scenario: Try claim buffer
- **WHEN** calling try_claim to obtain a buffer for zero-copy writing
- **THEN** a buffer handle is returned on success or back-pressure error on failure

### Requirement: Subscriber Trait
The library SHALL define an `AeronSubscriber` trait with methods for receiving messages from Aeron channels.

#### Scenario: Poll for messages
- **WHEN** calling the poll method with a message handler
- **THEN** available messages are delivered to the handler without blocking

#### Scenario: Non-blocking poll
- **WHEN** polling and no messages are available
- **THEN** the method returns immediately with a count of zero

### Requirement: Unified Error Handling
The library SHALL provide a common error type that unifies error conditions across all transport implementations.

#### Scenario: Transport error mapping
- **WHEN** a transport operation fails
- **THEN** it returns a `TransportError` with sufficient detail for debugging

#### Scenario: Error propagation
- **WHEN** a transport error occurs
- **THEN** the error can be handled uniformly regardless of the underlying implementation

#### Scenario: Back-pressure indication
- **WHEN** the transport cannot accept a message due to buffer fullness
- **THEN** a specific back-pressure error variant is returned

### Requirement: Zero-Copy Buffer Types
The library SHALL define buffer abstraction types with lifetime bounds that enable zero-copy message handling.

#### Scenario: Publication claim buffer
- **WHEN** claiming a buffer for publication
- **THEN** a mutable buffer reference is provided with lifetime guarantees preventing use-after-free

#### Scenario: Subscription fragment buffer
- **WHEN** receiving a message fragment
- **THEN** a read-only buffer reference is provided without copying message data

### Requirement: Mock Implementation Support
The library SHALL provide mock implementations of the transport traits to enable testing without requiring an Aeron media driver or real transport backends.

#### Scenario: Mock publisher for testing
- **WHEN** tests use a mock publisher implementation
- **THEN** publication operations can be verified without requiring Aeron dependencies

#### Scenario: Mock subscriber for testing
- **WHEN** tests use a mock subscriber implementation
- **THEN** subscription operations can be verified and controlled message delivery can be simulated

#### Scenario: Mockall integration
- **WHEN** using the mockall testing library
- **THEN** trait methods can be mocked with expectations and custom behavior

#### Scenario: In-memory mock behavior
- **WHEN** using the provided in-memory mock implementation
- **THEN** published messages can be inspected and injected messages can be consumed
