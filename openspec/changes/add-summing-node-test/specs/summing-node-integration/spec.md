## ADDED Requirements

### Requirement: Stateful Node Pattern
The library SHALL demonstrate a pattern for creating stateful Wingfoil nodes that poll Rusteron subscribers and maintain processing state.

#### Scenario: Node with state
- **WHEN** a SummingNode is created with initial state
- **THEN** it maintains a running sum across multiple cycle invocations

#### Scenario: State updates on message receipt
- **WHEN** the cycle() method receives an i64 message
- **THEN** the running sum is updated by adding the received value

#### Scenario: Multiple messages processed
- **WHEN** multiple i64 values are received across cycles
- **THEN** the sum accumulates correctly (e.g., 1+2+3+4+5 = 15)

### Requirement: Binary Message Parsing
The library SHALL demonstrate parsing binary message formats from Rusteron fragment buffers.

#### Scenario: Parse little-endian i64
- **WHEN** an 8-byte message is received
- **THEN** the bytes are interpreted as a little-endian i64 value

#### Scenario: Zero-copy parsing
- **WHEN** parsing i64 from fragment buffer
- **THEN** the value is read directly without intermediate copying

### Requirement: Wingfoil Integration
The library SHALL demonstrate registering a node with Wingfoil to receive cycle callbacks for polling.

#### Scenario: Node registration
- **WHEN** SummingNode is added to the Wingfoil graph
- **THEN** its cycle() method is invoked on each graph execution

#### Scenario: Polling in cycle
- **WHEN** cycle() is called by Wingfoil
- **THEN** the node polls the Rusteron subscriber non-blocking

#### Scenario: State observable
- **WHEN** the graph has executed
- **THEN** the final sum can be retrieved from the node

### Requirement: End-to-End Integration Test
The library SHALL provide an integration test demonstrating complete message flow with stateful processing.

#### Scenario: Publisher to subscriber flow
- **WHEN** i64 values are published via RusteronPublisher
- **THEN** SummingNode receives them via RusteronSubscriber polling

#### Scenario: Correct sum calculation
- **WHEN** values 1, 2, 3, 4, 5 are published
- **THEN** the final sum equals 15

#### Scenario: Message count tracking
- **WHEN** 5 messages are published
- **THEN** the node reports receiving 5 messages

### Requirement: Simple Demonstration
The integration test SHALL be minimal and focused on the core stateful processing pattern.

#### Scenario: Concise implementation
- **WHEN** reviewing the complete test code
- **THEN** it is under 150 lines including SummingNode

#### Scenario: Clear reference
- **WHEN** developers need to build stateful processors
- **THEN** the test provides a clear pattern for state management with Aeron input
