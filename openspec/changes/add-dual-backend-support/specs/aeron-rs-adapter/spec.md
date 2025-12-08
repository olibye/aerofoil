## MODIFIED Requirements

### Requirement: Feature Flag Configuration
The library SHALL support compile-time selection of the aeron-rs backend through a Cargo feature flag, which can be enabled alongside Rusteron.

#### Scenario: Feature flag enabled
- **WHEN** the `aeron-rs` feature is enabled in Cargo.toml
- **THEN** aeron-rs dependency is compiled and linked

#### Scenario: Feature flag disabled
- **WHEN** the `aeron-rs` feature is not enabled
- **THEN** no aeron-rs code or dependencies are included in the build

#### Scenario: Both backends enabled
- **WHEN** both `rusteron` and `aeron-rs` features are enabled
- **THEN** both modules are compiled and available
- **AND** benchmarks can compare both backends in a single run

#### Scenario: Single backend mode preserved
- **WHEN** only `aeron-rs` feature is enabled (without `rusteron`)
- **THEN** only aeron-rs code is compiled
