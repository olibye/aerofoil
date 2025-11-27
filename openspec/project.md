# Project Context

## Purpose
Rust library providing wingfoil aeron adapters

## Tech Stack
- Rust
- Wingfoil https://docs.rs/wingfoil/latest/wingfoil/
- Aeron https://github.com/aeron-io/aeron
- Rusteron aeron client wrapper https://github.com/gsrxyz/rusteron
- Pure rust aeron client https://github.com/UnitedTraders/aeron-rs
- Mockall for mocking in unit tests https://github.com/asomers/mockall
- Benchmarking https://docs.rs/criterion/latest/criterion/

## Project Conventions
- Wingfoil for message processing
- Aeron for input and output
- Zero copy message handling where possible
- Configuration object model the abstracts the configuration source
- Support both rusteron and aeron-rs clients with feature flags
- Use static dispatch in hot paths, ban dynamic traits in hot paths
- Document latency compromises between rusteron and aeron-rs
- Document cases of clone and copy with explanations
- Document design decisions using unit test cases
- Suggests improvements to the openspec meta prmopts after learning lessons from implementation
- No code in documents, simply add working code files and reference them from documentation
- Use idiomatic rust cargo examples instead of examples in modules
- Keep documentation for rusteron separate from aeron_rs documentation in their own modules

### Wingfoil Node Conventions
- Wingfoil nodes SHALL be single-threaded and execute within Wingfoil's graph execution context
- Node state SHALL NOT use thread-safe primitives (Arc<Mutex<>>, Arc<RwLock<>>, etc.) for internal processing
- Nodes communicate via Wingfoil streams and channels, not through shared memory
- Thread-safe wrappers (Arc<Mutex<>>) are ONLY acceptable for:
  - Test verification where external code needs to observe node state after graph execution
  - Interfacing with external multi-threaded systems outside the graph
- Production node implementations should expose state via Wingfoil's Stream trait, not Arc<Mutex<>>
- Keep node implementations simple and fast - avoid synchronization overhead in hot paths
### Lessons learned
- Document lessons learned in @openspec/lessons-learned/ consider these lessos in future designs

### Code Style
- Idiomatic Rust patterns
- Standard Rust formatting with `rustfmt`

### Architecture Patterns
- Support processor pinning to specific CPU cores for performance
- Aeron for all signals including logging and monitoring in production, with fallback to stdout/stderr in development
- Separate a module for a higher level SBE message abstraction over raw byte buffers

### Unit Testing Strategy
- Write tests inline using `#[cfg(test)] mod tests` at the bottom of each module
- NEVER create separate `tests.rs` files - use idiomatic inline test modules
- Use mockall for mocking in unit tests
- Prefer mockall's `#[automock]` for generating mocks from traits
- For traits where mockall has limitations (complex lifetimes, generic closures), provide manual test implementations inline
- Only expose mock objects in test configurations
- Validate examples in comments with doc tests
- Use the given when then style for unit tests
**For detailed mocking guidelines, see [mocking.md](mocking.md)**

### Integration Testing Strategy
- **NEVER write ignored tests** - Ignored tests prove nothing and rot over time
- Support Linux production and MacOS development environments
- Integration tests requiring external dependencies must be:
  - Self-contained (start/stop dependencies automatically)
  - Use feature flags to conditionally compile (not `#[ignore]`)
  - Fail fast with clear error messages if dependencies unavailable
  - Tests requiring the aeron media driver should start the driver or fail
- Use RAII guards for automatic resource cleanup (MediaDriverGuard pattern)
- Integration tests should be runnable via `cargo test` without manual setup
- Write integration tests in `tests/` directory at project root (standard Cargo convention)
- Use Given/When/Then structure for integration test clarity
- Ignore the java aeron media driver
- There is no homebrew tap for aeron/aeronmd
**For setting up the integration test environment (installing aeronmd, C++ toolchain), see [integration-test.md](integration-test.md)**

### Examples Strategy
- Use idiomatic Rust cargo examples in `examples/` directory at project root
- NEVER put example code in modules - use `examples/*.rs` files
- Examples should demonstrate real-world usage patterns
- Each example should be runnable via `cargo run --example <name>`
- Keep examples focused on single use cases

### Benchmarking Strategy
- Use criterion for benchmarking key code paths
- Combine examples into benchmarks where possible

### Git Workflow
- Task branches off main
- 
## Domain Context
- High frequency trading systems
- Stateful stream processing for position keeping

## Important Constraints
- The input and output code paths must be non-blocking and low latency
- Consider processor and OS specific optimisation for high performance. Maintain @possible-optimisations.md for future implementations 
## External Dependencies

### Aeron Media Driver
- **Purpose**: Required for Rusteron and aeron-rs transport backends to function
- **Build and Installation**:
  - git clone https://github.com/real-logic/aeron
  - cd aeron
  - ./cppbuild/cppbuild --build-aeron-driver --no-tests
- **Runtime requirement**: Media driver must be running before starting applications that use Aeron transports
- **Development**: Required for integration tests
- **Documentation**: https://github.com/real-logic/aeron/wiki

### C++ Toolchain (Rusteron only)
- **Purpose**: Required to compile rusteron crate and its C++ Aeron bindings
- **Components**: C++17 compatible compiler, CMake, standard build tools
- **Installation**: See [integration-test.md](integration-test.md) for platform-specific setup
