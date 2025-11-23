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

### Lessons learned
- Document lessons learned in @openspec/lessons-learned/ consider these lessos in future designs

### Code Style
- Idiomatic Rust patterns
- Standard Rust formatting with `rustfmt`

### Architecture Patterns
- Support processor pinning to specific CPU cores for performance
- Aeron for all signals including logging and monitoring in production, with fallback to stdout/stderr in development
- Separate a module for a higher level SBE message abstraction over raw byte buffers

### Testing Strategy
- Use mockall for mocking in unit tests
- Prefer mockall's `#[automock]` for generating mocks from traits
- For traits where mockall has limitations (complex lifetimes, generic closures), provide manual test implementations
- Only expose mock objects in test configurations
- Validate examples in comments with doc tests
- Add unit tests in line with implementation
- Use the given when then style for unit tests

**For detailed mocking guidelines, see [mocking.md](mocking.md)**

**For integration testing with external dependencies, see [integration-test.md](integration-test.md)**
- Support Linux production and MacOS development environments

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

## External Dependencies

### Aeron Media Driver
- **Purpose**: Required for Rusteron and aeron-rs transport backends to function
- **Installation**:
  - Download from https://github.com/real-logic/aeron/releases
  - Or install via package manager (varies by platform)
- **Runtime requirement**: Media driver must be running before starting applications that use Aeron transports
- **Development**: Required for integration tests
- **Documentation**: https://github.com/real-logic/aeron/wiki

### C++ Toolchain (Rusteron only)
- **Purpose**: Required to compile rusteron crate and its C++ Aeron bindings
- **Components**:
  - C++17 compatible compiler (Clang from Xcode)
  - CMake (for building Aeron C++ libraries)
  - Standard C++ build tools
- **macOS Installation**:
  ```bash
  # Install Xcode Command Line Tools
  xcode-select --install

  # Install CMake via Homebrew
  brew install cmake

  # Verify installation
  clang++ --version
  cmake --version
  ```
- **Not required**: When using `aeron-rs` feature (pure Rust)
- **Documentation**: See rusteron installation guide

### Java Runtime (Optional)
- **Purpose**: Aeron Media Driver can be run as Java application
- **Requirement**: JDK 8 or later
- **Alternative**: Use embedded media driver or C++ media driver
- **Documentation**: https://github.com/real-logic/aeron
