# GitHub Actions CI/CD

## ADDED Requirements

### Requirement: CI Workflow Triggers
The CI workflow SHALL run automatically on push and pull request events to the main branch.

#### Scenario: Push triggers CI
- **WHEN** a push is made to the main branch
- **THEN** the CI workflow is triggered and runs all checks

#### Scenario: PR triggers CI
- **WHEN** a pull request is opened or updated targeting the main branch
- **THEN** the CI workflow is triggered and runs all checks

### Requirement: Code Formatting Check
The CI workflow SHALL verify code formatting using rustfmt.

#### Scenario: Formatting verification
- **WHEN** `cargo fmt --check` is executed in CI
- **THEN** the check passes if all code follows rustfmt standards
- **AND** the check fails if any file is not properly formatted

### Requirement: Linting Check
The CI workflow SHALL run clippy with the embedded-driver feature to detect code issues.

#### Scenario: Clippy linting
- **WHEN** `cargo clippy --features embedded-driver -- -D warnings` is executed in CI
- **THEN** the check passes if there are no warnings or errors
- **AND** the check fails if clippy detects any issues

### Requirement: Test Execution
The CI workflow SHALL run tests with the embedded-driver feature.

#### Scenario: Test execution
- **WHEN** `cargo test --features embedded-driver` is executed in CI
- **THEN** all tests are run
- **AND** the check passes if all tests pass

### Requirement: Build Verification
The CI workflow SHALL verify the project builds successfully.

#### Scenario: Project build
- **WHEN** `cargo build --features embedded-driver` is executed in CI
- **THEN** the build completes without errors

### Requirement: Benchmark Build Verification
The CI workflow SHALL verify benchmarks compile with both standard and dhat-heap features.

#### Scenario: Standard benchmarks compile
- **WHEN** `cargo build --benches --features embedded-driver` is executed in CI
- **THEN** all benchmarks compile successfully

#### Scenario: Allocation tracking benchmarks compile
- **WHEN** `cargo build --benches --features embedded-driver,dhat-heap` is executed in CI
- **THEN** the allocation tracking benchmark compiles with dhat support enabled
