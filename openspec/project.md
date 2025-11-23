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
[Document key external services, APIs, or systems]
