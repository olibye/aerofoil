# Tasks

## Phase 1: Create Example Files

- [x] Create `examples/subscriber_node_reference_access.rs` demonstrating AeronSubscriberValueRefNode with StreamPeekRef pattern
- [x] Create `examples/subscriber_node_value_access.rs` demonstrating AeronSubscriberValueNode with StreamPeek pattern
- [x] Create `examples/counting_node_composition.rs` showing complete counting node integration with Aeron
- [x] Create `examples/summing_node_composition.rs` showing complete summing node integration with Aeron
- [x] Verify all new examples compile: `cargo build --examples`
- [x] Verify all new examples can be listed: `cargo run --example` (without name shows list)

## Phase 2: Update Documentation References

- [x] Update `src/nodes/mod.rs` module doc comments to reference example files instead of embedding code
- [x] Update `src/nodes/subscriber.rs` doc comments for `AeronSubscriberValueRefNode` to reference examples
- [x] Update `src/nodes/subscriber.rs` doc comments for `AeronSubscriberValueNode` to reference examples
- [x] Update `tests/common/mod.rs` doc comments to reference examples instead of embedding code
- [x] Update `tests/counting_node_value_test.rs` module comments to reference examples and remove embedded code
- [x] Update `tests/summing_node_test.rs` module comments to reference examples and remove embedded code

## Phase 3: Move Setup Instructions

- [x] Verify `openspec/integration-test.md` contains comprehensive setup instructions for running integration tests
- [x] Update test module comments to reference `openspec/integration-test.md` for setup instructions
- [x] Remove bash command examples from test file comments (keep references to integration-test.md)

## Phase 4: Validation

- [x] Run `rg "^\s*///.*\`\`\`rust" src/ tests/` to verify no Rust code blocks remain in doc comments
- [x] Run `rg "^\s*//!.*\`\`\`rust" src/ tests/` to verify no Rust code blocks remain in module comments
- [x] Run `cargo doc --no-deps` to verify documentation builds correctly with example references
- [x] Run `cargo test --doc` to verify any remaining doctests still pass
- [x] Update `openspec/project.md` conventions if needed to clarify the examples-only approach
- [x] Run `cargo fmt` on all modified files
- [x] Run `cargo clippy` to ensure no new warnings

## Phase 5: Documentation

- [x] Verify all example files have proper file-level documentation explaining what they demonstrate
- [x] Ensure examples follow the same code style as the rest of the project
