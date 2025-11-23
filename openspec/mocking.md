# Mocking Strategy

Guidelines for test mocking in the aerofoil project.

## Core Testing Principles

1. **NEVER write ignored tests** - Ignored tests prove nothing and rot over time
2. **Unit tests should not require external dependencies** - Use mocks/manual implementations
3. **Integration tests must be self-contained** - Automatically start/stop dependencies
4. **All tests must be runnable via `cargo test`** - No manual setup required

## When to Use Mockall

Use mockall's `#[automock]` attribute for traits with:

- **Simple method signatures**: No complex lifetime parameters
- **No generic closures**: No `FnMut` or `Fn` parameters in trait methods
- **Standard return types**: Simple types, `Result<T, E>`, or boxed futures
- **No associated lifetimes in generics**: Method generics don't reference `self`'s lifetime

**Example traits suitable for mockall:**
- Simple getters/setters
- Configuration interfaces
- I/O abstractions without lifetime constraints
- State management interfaces

See `#[automock]` usage in `#[cfg(test)] mod tests` blocks within modules for working examples.

## When NOT to Use Mockall

Provide **manual test implementations** for traits with:

- **Explicit lifetime parameters**: Methods like `fn claim<'a>(&'a mut self) -> Buffer<'a>`
- **Generic closure parameters**: Methods like `fn poll<F: FnMut(&Item)>(&mut self, handler: F)`
- **Complex associated types with lifetimes**: GATs with lifetime constraints
- **High-rank trait bounds**: Methods with `for<'a>` syntax

### Why Mockall Fails on These

- Mockall's procedural macro cannot generate correct lifetime relationships
- Generic closures create lifetime constraints mockall can't express
- Results in compilation errors about lifetime requirements

**Example traits requiring manual implementation:**
- `AeronPublisher` (has `try_claim<'a>` with lifetime-bound return)
- `AeronSubscriber` (has `poll<F: FnMut>` with generic closure)
- Buffer management traits with lifetime-bound views

See `#[cfg(test)] mod tests` blocks in transport modules for working manual test implementation examples.

## Guidelines for Manual Test Implementations

When mockall is not suitable:

1. **Keep it simple**: Manual implementations should be ~10-30 lines
2. **Store test data**: Use `Vec`, `VecDeque`, or similar collections to capture calls
3. **Add helper methods**: Provide inspection methods for assertions (e.g., `messages()`)
4. **Document the pattern**: Add comments explaining why manual implementation is needed
5. **Provide examples**: Include manual implementations in `tests.rs` as reference

### Common Patterns

**Capturing Method Calls:**
- Store inputs in a `Vec` or `VecDeque`
- Track state changes
- Return predictable test values

**Injecting Test Data:**
- Provide helper methods to inject test messages
- Pop from queue during trait method calls
- Return injected data to test code

**Example locations:**
- Look for `#[cfg(test)] mod tests` at the bottom of each module
- Manual test implementations are inline with the code they test

## Decision Tree

**Does the trait have any of these?**
- Methods with explicit lifetime parameters like `<'a>`
- Methods with generic closures like `<F: FnMut>`
- GATs or complex associated types

**YES** → Use manual test implementation
- Write inline in `#[cfg(test)] mod tests` blocks
- Keep implementation simple (10-30 lines)

**NO** → Use mockall's `#[automock]`
- Add `#[cfg_attr(test, automock)]` to trait
- Use `Mock*` types in tests

## References

- Working examples: Look for `#[cfg(test)] mod tests` blocks in source modules
- Mockall documentation: https://github.com/asomers/mockall
- Mockall limitations: https://github.com/asomers/mockall/blob/master/mockall/examples/limitations.md
