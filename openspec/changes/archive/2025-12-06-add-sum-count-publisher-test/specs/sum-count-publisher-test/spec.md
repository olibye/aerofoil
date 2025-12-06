# sum-count-publisher-test Specification

## Purpose

Demonstrates the fan-out pattern and publisher-in-callback idiom: a single input subscriber feeds multiple processing nodes, each publishing to separate Aeron output streams via their output callbacks.

## ADDED Requirements

### Requirement: Fan-Out Pattern with Shared Upstream

The integration tests SHALL demonstrate multiple nodes consuming from a single upstream source.

#### Scenario: Given single subscriber when multiple nodes consume then both receive values

- **GIVEN** an `AeronSubscriberValueNode` subscribed to input stream
- **AND** a `SummingNode` using the subscriber as upstream
- **AND** a `CountingNode` using the same subscriber as upstream
- **WHEN** the graph runs
- **THEN** both nodes process the same input values

### Requirement: Publisher-in-Callback Pattern

The integration tests SHALL demonstrate publishing to Aeron from node output callbacks.

#### Scenario: Given node with publisher callback when value changes then publishes to Aeron

- **GIVEN** a `SummingNode` with callback that captures an `AeronPublisher`
- **WHEN** the node processes a new value
- **THEN** the callback publishes the running sum via `publisher.offer()`

#### Scenario: Given output streams when graph completes then contain expected values

- **GIVEN** input values [1, 2, 3, 4, 5] published to stream 2001
- **AND** `SummingNode` publishing to stream 2002
- **AND** `CountingNode` publishing to stream 2003
- **WHEN** the graph runs for sufficient cycles
- **THEN** stream 2002 contains the sum (15)
- **AND** stream 2003 contains the count (5)
