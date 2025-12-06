# documentation-conventions Specification

## Purpose
TBD - created by archiving change remove-code-from-comments. Update Purpose after archive.
## Requirements
### Requirement: Code Examples in Separate Files

All Rust code examples MUST be placed in runnable example files in the `examples/` directory, not embedded in comments. Embedded code examples cannot be verified by the compiler, are hard to discover, and don't follow idiomatic Rust cargo conventions. Runnable examples serve as both documentation and integration tests.

#### Scenario: Developer writes documentation for a new node type

- **WHEN** a developer documents a new Wingfoil node implementation and wants to show usage examples
- **THEN** they create a file in `examples/` (e.g., `examples/my_node_usage.rs`)
- **AND** reference it in doc comments with text like "See `examples/my_node_usage.rs` for a complete example"
- **AND** do not embed code blocks with ````rust` or ````rust,ignore` in the doc comments

#### Scenario: Existing documentation contains embedded code

- **WHEN** reviewing code that contains doc comments with ````rust,ignore` code blocks
- **THEN** the embedded code is extracted to `examples/`
- **AND** the doc comment is updated to reference the example file
- **AND** the code block is removed from the comment

### Requirement: Bash Setup Instructions Location

Environment setup instructions involving bash commands MUST be documented in `openspec/integration-test.md` or similar markdown documentation files, not in source code comments. Setup instructions are environmental rather than code-specific, and centralizing them makes onboarding easier while preventing duplication.

#### Scenario: Integration test requires media driver setup

- **WHEN** documenting requirements for an integration test that needs the Aeron media driver running
- **THEN** setup instructions are placed in `openspec/integration-test.md`
- **AND** the test module doc comment may reference that documentation
- **AND** bash command examples are not included directly in the test file

### Requirement: Doc Comment References to Examples

When documentation needs to illustrate usage, it MUST reference example files by path rather than embedding code. This provides a single source of truth for working examples, allows the compiler to verify examples stay current with API changes, and enables users to run examples to learn the API.

#### Scenario: API documentation shows how to use a function

- **WHEN** writing doc comments for a public function that needs usage documentation
- **THEN** the comment may include a brief prose description of usage
- **AND** must reference a complete working example like "For a complete example, see `examples/example_name.rs`"
- **AND** must not include ````rust` code blocks showing example usage

#### Scenario: Small API usage snippet in doctests

- **WHEN** a function has a simple, self-contained usage pattern demonstrable in 1-5 lines
- **THEN** a doctest may be used (``````` in doc comments that runs as part of `cargo test`)
- **AND** more complex scenarios still use separate example files
- **AND** doctests actually compile and run (do not use `ignore`)

