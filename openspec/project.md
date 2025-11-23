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

## Project Conventions
- Wingfoil for message processing
- Aeron for input and output
- Zero copy message handling where possible
- Configuration object model the abstracts the configuration source

### Code Style
- Idiomatic Rust patterns
- Standard Rust formatting with `rustfmt`

### Architecture Patterns
- Support processor pinning to specific CPU cores for performance
- Aeron for all signals including logging and monitoring

### Testing Strategy
- Unit tests with mockall for mocking

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
