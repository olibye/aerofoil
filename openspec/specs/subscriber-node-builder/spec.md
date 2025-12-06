# subscriber-node-builder Specification

## Purpose
TBD - created by archiving change add-subscriber-node-builder. Update Purpose after archive.
## Requirements
### Requirement: Subscriber Node Builder

The library SHALL provide an `AeronSubscriberNodeBuilder` that handles `Rc<RefCell<>>` wrapping.

#### Scenario: Given builder when all fields set then builds valid node

- **GIVEN** an `AeronSubscriberNodeBuilder` with subscriber, parser, and default value set
- **WHEN** `build()` is called
- **THEN** returns `Rc<RefCell<AeronSubscriberValueNode<T, F, S>>>`
- **AND** the node can be cloned for the graph (coerces to `Rc<dyn Node>`)
- **AND** the node can be used directly by downstream nodes

#### Scenario: Given builder when build_ref called then builds ValueRefNode

- **GIVEN** an `AeronSubscriberNodeBuilder` with subscriber, parser, and default value set
- **WHEN** `build_ref()` is called
- **THEN** returns `Rc<RefCell<AeronSubscriberValueRefNode<T, F, S>>>`

#### Scenario: Given builder when required field missing then panics with clear message

- **GIVEN** an `AeronSubscriberNodeBuilder` without subscriber set
- **WHEN** `build()` is called
- **THEN** panics with message indicating missing subscriber

### Requirement: Fluent Builder API

The builder SHALL provide a fluent API for configuration.

#### Scenario: Given builder when chaining methods then returns self

- **GIVEN** an `AeronSubscriberNodeBuilder`
- **WHEN** `.subscriber(s).parser(p).default(v)` is called
- **THEN** each method returns `Self` allowing method chaining

#### Scenario: Given node type when builder called then returns builder

- **GIVEN** the `AeronSubscriberValueNode` type
- **WHEN** `AeronSubscriberValueNode::builder()` is called
- **THEN** returns an `AeronSubscriberNodeBuilder` instance

### Requirement: Type-Safe Builder

The builder SHALL enforce type safety at compile time.

#### Scenario: Given parser returning Option of T when built then node type is T

- **GIVEN** a parser `|bytes| -> Option<i64>`
- **WHEN** builder is constructed with this parser
- **THEN** the resulting node has type `AeronSubscriberValueNode<i64, _, _>`

#### Scenario: Given mismatched default type when built then compile error

- **GIVEN** a parser `|bytes| -> Option<i64>` and default value `"string"`
- **WHEN** builder is constructed
- **THEN** compilation fails with type mismatch error

### Requirement: Ergonomic Test Usage

The builder SHALL simplify integration test setup.

#### Scenario: Given test setup when using builder then reduces boilerplate

- **GIVEN** a test requiring Aeron subscriber node in Wingfoil graph
- **WHEN** using the builder pattern
- **THEN** setup requires fewer lines
- **AND** intent is clearer (no manual Rc/RefCell manipulation)

```rust
// Before (4 lines, manual wrapping):
let value_node = AeronSubscriberValueNode::new(subscriber, parser, 0i64);
let value_node_rc = Rc::new(RefCell::new(value_node));
let upstream_ref = value_node_rc.clone();
let graph_node: Rc<dyn Node> = value_node_rc;

// After (builder handles wrapping, clone for graph):
let node = AeronSubscriberValueNode::builder()
    .subscriber(subscriber)
    .parser(parser)
    .default(0i64)
    .build();

// Clone for graph (coerces to Rc<dyn Node>), use directly for upstream
let graph = Graph::new(vec![node.clone(), downstream], ...);
let downstream = MyNode::new(node, callback);
```

