## MODIFIED Requirements
### Requirement: Feature Flag Configuration
The library SHALL support compile-time selection of the aeron-rs backend through a Cargo feature flag, allowing coexistence with Rusteron.

#### Scenario: Feature flag enabled
- **WHEN** the `aeron-rs` feature is enabled in Cargo.toml
- **THEN** aeron-rs dependency is compiled and linked

#### Scenario: Feature flag disabled
- **WHEN** the `aeron-rs` feature is not enabled
- **THEN** no aeron-rs code or dependencies are included in the build

#### Scenario: Coexistence
- **WHEN** both `rusteron` and `aeron-rs` features are enabled simultaneously
- **THEN** both backends are available for use (e.g., for benchmarking comparison)
