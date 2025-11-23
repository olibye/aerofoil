# Project Context

## Purpose
Rust library providing wingfoil aeron adapters

## Tech Stack
- Rust
- Wingfoil https://docs.rs/wingfoil/latest/wingfoil/
- Aeron https://github.com/aeron-io/aeron
- Rusteron aeron client wrapper https://github.com/gsrxyz/rusteron
- Pure rust aeron client https://github.com/UnitedTraders/aeron-rs
- Mock object test library https://github.com/asomers/mockall
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

### Code Style
- Idiomatic Rust patterns
- Standard Rust formatting with `rustfmt`

### Architecture Patterns
- Support processor pinning to specific CPU cores for performance
- Aeron for all signals including logging and monitoring in production, with fallback to stdout/stderr in development

### Testing Strategy
- Unit tests with mockall for mocking

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
