# Development Guide: Aerofoil

## Prerequisites

- **Rust:** Latest stable version (managed via `rustup`)
- **Aeron Media Driver:** Required for integration tests
- **C++ Toolchain:** Required for rusteron (CMake, C++17 compiler)

## Build and Run

```bash
cd aerofoil/

# Build (default features - rusteron)
cargo build

# Build with all backends
cargo build --all-features

# Run a specific example
cargo run --example <name>
```

## Testing

### Unit Tests

```bash
cargo test
```

Unit tests are inline using `#[cfg(test)] mod tests` at the bottom of each module. Never create separate `tests.rs` files.

### Integration Tests

```bash
# Requires aeronmd to be installed (see below)
cargo test --all-features
```

Integration tests live in `tests/` directory (standard Cargo convention). They are self-contained and automatically start/stop dependencies using `MediaDriverGuard`.

### Benchmarks

```bash
cargo bench
```

Uses Criterion for statistical benchmarking of key code paths.

## Integration Test Environment Setup

### Installing Aeron Media Driver (aeronmd)

#### macOS

**Option 1: Download Binary (Recommended)**
1. Download from https://github.com/real-logic/aeron/releases
2. Extract the archive
3. Copy `lib/aeronmd` to `/usr/local/bin/aeronmd`
4. Make executable: `chmod +x /usr/local/bin/aeronmd`
5. Verify: `which aeronmd`

**Option 2: Build from Source**
1. Install Xcode Command Line Tools: `xcode-select --install`
2. Install CMake: `brew install cmake`
3. Clone: `git clone https://github.com/real-logic/aeron.git`
4. Build: `cd aeron && ./gradlew`
5. Copy `./cppbuild/Release/binaries/aeronmd` to `/usr/local/bin/`

#### Linux

1. Download from https://github.com/real-logic/aeron/releases
2. Extract and copy `lib/aeronmd` to `/usr/local/bin/aeronmd`
3. `chmod +x /usr/local/bin/aeronmd`

### Installing C++ Toolchain (Rusteron Only)

#### macOS
```bash
xcode-select --install
brew install cmake
```

#### Linux
```bash
sudo apt-get install build-essential cmake
```

### Verification

```bash
which aeronmd                      # Should show path
cargo build --features rusteron    # Build check
cargo test --features rusteron     # Run all tests
```

### Troubleshooting

- **aeronmd not found**: Check `which aeronmd`, ensure `/usr/local/bin` in PATH
- **C++ build failures**: Verify Xcode CLT (macOS) or build-essential (Linux)
- **Port conflicts**: `pkill aeronmd` then `ps aux | grep aeronmd`

## Mocking Strategy

### Decision Tree

Does the trait have explicit lifetime parameters (`<'a>`), generic closures (`<F: FnMut>`), or complex GATs?

- **YES** -> Use manual test implementation (inline in `#[cfg(test)] mod tests`)
- **NO** -> Use mockall's `#[automock]` / `#[cfg_attr(test, automock)]`

### When to Use Mockall

Traits with simple method signatures, no complex lifetime parameters, no generic closures, standard return types:
- Simple getters/setters
- Configuration interfaces
- I/O abstractions without lifetime constraints

### When NOT to Use Mockall (Manual Test Implementations)

Traits where mockall's proc macro cannot generate correct lifetime relationships:
- `AeronPublisher` (has `try_claim<'a>` with lifetime-bound return)
- `AeronSubscriber` (has `poll<F: FnMut>` with generic closure)
- Buffer management traits with lifetime-bound views

### Manual Test Implementation Guidelines

1. Keep implementations ~10-30 lines
2. Store test data in `Vec` or `VecDeque` to capture calls
3. Add helper inspection methods for assertions (e.g., `messages()`)
4. Document why manual implementation is needed
5. Implementations live inline in `#[cfg(test)] mod tests` blocks

## Testing Conventions

### Unit Testing

- Write tests inline using `#[cfg(test)] mod tests` at the bottom of each module
- NEVER create separate `tests.rs` files
- Use Given/When/Then style for test names
- Validate examples in comments with doc tests
- Only expose mock objects in test configurations

### Integration Testing

- **NEVER write ignored tests** - ignored tests prove nothing and rot over time
- Tests requiring external dependencies must be self-contained (start/stop automatically)
- Use feature flags to conditionally compile (not `#[ignore]`)
- Fail fast with clear error messages if dependencies unavailable
- Tests requiring the aeron media driver should start the driver or fail
- Use RAII guards for automatic resource cleanup (`MediaDriverGuard` pattern)
- Use Given/When/Then structure for clarity

### Shared Test Helpers (`tests/common/mod.rs`)

ALL common test code MUST be in `tests/common/mod.rs`:
- `MediaDriverGuard` - RAII guard for Aeron media driver lifecycle
- `SummingNode` - Example node demonstrating reference-based access (`StreamPeekRef`)
- `CountingNode` - Example node demonstrating value-based access (`StreamPeek`)

Import: `mod common; use common::{MediaDriverGuard, ...};`

### Testing Patterns with Wingfoil

**Observing Node State**: Use callback closures with `Rc<RefCell<>>` (not `Arc<Mutex<>>`):
- Wingfoil graphs execute in a single thread
- `Rc<RefCell<>>` provides interior mutability with runtime borrow checking
- No synchronization overhead

**CallBackStream**: For INPUT only, not output collection. Feeds test data into a graph at specified times.

**Interior Mutability**: Wingfoil nodes wrapped in `RefCell` for `cycle(&mut self)` calls via `Rc<dyn Node>`.

## Code Style & Conventions

### Examples Strategy

- Use idiomatic Rust cargo examples in `examples/` directory
- NEVER put example code in comments - use `examples/*.rs` files
- Each example runnable via `cargo run --example <name>`
- Keep examples focused on single use cases

### Documentation Conventions

- Document design decisions using unit test cases
- Document latency compromises between rusteron and aeron-rs
- Document cases of clone and copy with explanations
- Never add code in markup documentation - reference working code files instead
- Keep rusteron documentation separate from aeron-rs documentation in their own modules
- Doctests allowed only for simple 1-5 line patterns

### General

- Idiomatic Rust patterns with standard `rustfmt` formatting
- Prefer simple functions to macros where possible
- Factor out common code into helper functions even for tests
- Minimise dependencies where possible
- Use static dispatch in hot paths, ban dynamic traits in hot paths

## CI/CD

GitHub Actions workflow (`.github/workflows/rust.yml`):
- Formatting check: `cargo fmt --check`
- Linting: `cargo clippy`
- Test execution
- Build verification
- Benchmark build with `embedded-driver` and `dhat-heap` features

### CI Media Driver

Aeron MD runs in SHARED threading mode with UUID-based `/dev/shm` sandboxing for CI environments.
