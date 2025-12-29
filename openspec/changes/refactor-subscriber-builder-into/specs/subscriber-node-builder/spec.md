## ADDED Requirements
### Requirement: Into-based Default Value
The builder SHALL accept any type convertible to `T` for the default value.

#### Scenario: Given value convertible to T when default called then accepts value
- **GIVEN** a builder for type `T = i64`
- **WHEN** `.default(42i32)` is called (assuming i32 implements Into<i64>)
- **THEN** the builder accepts the value and converts it

### Requirement: Into-based Subscriber
The builder SHALL accept any type convertible to `S` for the subscriber.

#### Scenario: Given subscriber convertible to S when subscriber called then accepts value
- **GIVEN** a builder for subscriber `S`
- **WHEN** `.subscriber(v)` where `v: Into<S>` is called
- **THEN** it compiles and sets the subscriber

### Requirement: Fluent Factory Method
The library SHALL provide a fluent factory method `receive_aeron` that simplifies node creation.

#### Scenario: Given subscriber and parameters when receive_aeron called then returns builder
- **GIVEN** an Aeron subscriber `sub` (or convertible)
- **WHEN** `receive_aeron(sub)` is called
- **THEN** it returns an `AeronSubscriberNodeBuilder`
- **AND** the subscriber is set in the builder
