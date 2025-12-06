# Remove Code from Comments

## Why

The codebase currently contains Rust code examples embedded in comments throughout multiple files, violating the project convention stated in `openspec/project.md:95-96`:

> NEVER put example code in comments - use `examples/*.rs` files.
> Create openspec tasks to check verify that code and function comments do not include rust code.

### Current Violations

Code blocks with ````rust,ignore` and ````bash` markers were found in:

1. **src/nodes/mod.rs** - Module-level documentation contains multiple code examples
2. **src/nodes/subscriber.rs** - Multiple doc comment sections with embedded code
3. **tests/common/mod.rs** - Test helper documentation with code examples
4. **tests/counting_node_value_test.rs** - Module doc comments with setup examples
5. **tests/summing_node_test.rs** - Module doc comments with setup examples

These embedded examples make it harder to:
- Verify that examples actually compile and run
- Keep examples synchronized with API changes
- Discover available examples (they're scattered across source files)
- Follow the project's stated conventions

## What Changes

1. **Extract code examples to `examples/` directory** - Create proper Cargo examples that can be run with `cargo run --example <name>`
2. **Replace code blocks with references** - Update doc comments to reference the example files instead of embedding code
3. **Maintain bash setup instructions separately** - Move environment setup instructions to `openspec/integration-test.md` or similar documentation files
4. **Update documentation conventions spec** - Formalize the no-code-in-comments policy in specs

## Impact

- Affected specs: `documentation-conventions` (new capability - MODIFIED requirements)
- Affected code: `src/nodes/mod.rs`, `src/nodes/subscriber.rs`, `tests/common/mod.rs`, test module files
- New files: 4 example files in `examples/` directory
- Dependencies: None - purely documentation and code organization
- User value: Examples are discoverable, verifiable, and runnable; documentation is more maintainable
- Testing: All examples must compile and be verifiable via `cargo build --examples`
