# transport-traits Specification

## Purpose
Defines the foundational trait-based abstractions for Aeron transport operations, enabling zero-cost polymorphism across different Aeron client implementations (Rusteron, aeron-rs) and test implementations. These traits establish the stable API contract for all transport adapters while supporting zero-copy message handling and non-blocking operations required for high-frequency trading systems.
## Requirements
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

### Requirement: Implementation Encapsulation
Transport adapter implementations SHALL NOT expose their inner wrapped types through public methods.

#### Scenario: No inner access methods
- **WHEN** implementing a transport adapter (e.g., `RusteronPublisher`, `AeronRsPublisher`)
- **THEN** the implementation SHALL NOT provide `inner()`, `as_inner()`, or similar methods that expose the wrapped backend type

#### Scenario: Backend-agnostic usage
- **WHEN** application code uses a transport adapter
- **THEN** it interacts only through the `AeronPublisher` or `AeronSubscriber` trait methods

#### Scenario: Encapsulation rationale
- **WHEN** users need backend-specific functionality
- **THEN** they should construct and use the backend type directly, not through the adapter wrapper

### Requirement: Test Implementation Support
The library SHALL design traits to be easily implementable for testing purposes without requiring an Aeron media driver.

#### Scenario: Simple test implementations
- **WHEN** implementing transport traits for testing
- **THEN** tests can create minimal implementations without Aeron dependencies

#### Scenario: Trait-based testing
- **WHEN** writing tests using generic trait bounds
- **THEN** test doubles can be substituted for real implementations

#### Scenario: Error simulation
- **WHEN** test implementations return specific errors
- **THEN** error handling code paths can be tested

