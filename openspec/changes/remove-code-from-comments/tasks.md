# Tasks

## Phase 1: Create Example Files

- [ ] Create `examples/subscriber_node_reference_access.rs` demonstrating AeronSubscriberNode with StreamPeekRef pattern
- [ ] Create `examples/subscriber_node_value_access.rs` demonstrating AeronSubscriberValueNode with StreamPeek pattern
- [ ] Create `examples/counting_node_composition.rs` showing complete counting node integration with Aeron
- [ ] Create `examples/summing_node_composition.rs` showing complete summing node integration with Aeron
- [ ] Verify all new examples compile: `cargo build --examples`
- [ ] Verify all new examples can be listed: `cargo run --example` (without name shows list)

## Phase 2: Update Documentation References

- [ ] Update `src/nodes/mod.rs` module doc comments to reference example files instead of embedding code
- [ ] Update `src/nodes/subscriber.rs` doc comments for `AeronSubscriberNode` to reference examples
- [ ] Update `src/nodes/subscriber.rs` doc comments for `AeronSubscriberValueNode` to reference examples
- [ ] Update `tests/common/mod.rs` doc comments to reference examples instead of embedding code
- [ ] Update `tests/counting_node_value_test.rs` module comments to reference examples and remove embedded code
- [ ] Update `tests/summing_node_test.rs` module comments to reference examples and remove embedded code

## Phase 3: Move Setup Instructions

- [ ] Verify `openspec/integration-test.md` contains comprehensive setup instructions for running integration tests
- [ ] Update test module comments to reference `openspec/integration-test.md` for setup instructions
- [ ] Remove bash command examples from test file comments (keep references to integration-test.md)

## Phase 4: Validation

- [ ] Run `rg "^\s*///.*\`\`\`rust" src/ tests/` to verify no Rust code blocks remain in doc comments
- [ ] Run `rg "^\s*//!.*\`\`\`rust" src/ tests/` to verify no Rust code blocks remain in module comments
- [ ] Run `cargo doc --no-deps` to verify documentation builds correctly with example references
- [ ] Run `cargo test --doc` to verify any remaining doctests still pass
- [ ] Run all examples to ensure they work: `for ex in examples/*.rs; do cargo run --example $(basename $ex .rs); done` (may need feature flags)
- [ ] Update `openspec/project.md` conventions if needed to clarify the examples-only approach
- [ ] Run `cargo fmt` on all modified files
- [ ] Run `cargo clippy` to ensure no new warnings

## Phase 5: Documentation

- [ ] Update `CHANGELOG.md` or similar to note the documentation reorganization
- [ ] Verify all example files have proper file-level documentation explaining what they demonstrate
- [ ] Ensure examples follow the same code style as the rest of the project
