# Integration Test Environment Setup

Instructions for installing external dependencies required to run integration tests.

**Note**: For how to write integration tests, see [project.md](project.md#integration-testing-strategy). Tests automatically manage dependencies using MediaDriverGuard.

## Required Dependencies

### 1. Aeron Media Driver (aeronmd)

Required for all Aeron integration tests. Tests will automatically start/stop it.

#### macOS Installation

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
6. Verify: `which aeronmd`

#### Linux Installation

1. Download from https://github.com/real-logic/aeron/releases
2. Extract the archive
3. Copy `lib/aeronmd` to `/usr/local/bin/aeronmd`
4. Make executable: `chmod +x /usr/local/bin/aeronmd`
5. Verify: `which aeronmd`

### 2. C++ Toolchain (Rusteron Only)

Required for building and testing the Rusteron adapter.

#### macOS

1. Install Xcode Command Line Tools: `xcode-select --install`
2. Install CMake: `brew install cmake` (Ensure version 3.30+)
3. Verify: `clang++ --version` and `cmake --version`

#### Linux

1. Install build tools and dependencies:
   ```bash
   sudo apt-get update
   sudo apt-get install -y build-essential libbsd-dev uuid-dev
   ```
2. Install CMake 3.30+ (Required):
   ```bash
   wget https://github.com/Kitware/CMake/releases/download/v3.31.5/cmake-3.31.5-linux-x86_64.sh -q -O /tmp/cmake-install.sh
   chmod +x /tmp/cmake-install.sh
   sudo /tmp/cmake-install.sh --prefix=/usr/local --skip-license
   ```
3. Verify: `g++ --version` and `cmake --version` (must be >= 3.30)

## Verification

After installation:
1. Run `which aeronmd` - should show path to binary
2. Run `cargo build --features embedded-driver` - build script checks dependencies
3. Run `cargo test --features embedded-driver` - runs all tests including integration

The build script will warn if aeronmd is not found.

## Troubleshooting

**aeronmd not found**
- Check `which aeronmd` returns a path
- Ensure `/usr/local/bin` is in your PATH
- Create symlink if needed: `sudo ln -s /path/to/aeronmd /usr/local/bin/aeronmd`

**C++ build failures**
- Verify Xcode Command Line Tools installed (macOS)
- Verify build-essential installed (Linux)
- Check `clang++ --version` or `g++ --version` works
- Try `cargo clean && cargo build --features embedded-driver`

**Port conflicts during tests**
- Kill existing processes: `pkill aeronmd`
- Verify: `ps aux | grep aeronmd`

## Continuous Integration

**macOS**: Install cmake, download and install aeronmd binary to `/usr/local/bin/`

**Linux**: Install cmake and build-essential, download and install aeronmd binary to `/usr/local/bin/`

See GitHub Actions workflows in `.github/workflows/` for complete examples.

## Testing Patterns with Wingfoil

### Observing Node State During Tests

Use **callback closures with Rc<RefCell<>>** to observe node state, following Wingfoil's single-threaded pattern.

**Why Rc<RefCell<>> not Arc<Mutex<>>?**
- Wingfoil graphs execute in a single thread
- `Rc<RefCell<>>` provides interior mutability with runtime borrow checking
- No synchronization overhead (better performance)
- Follows Wingfoil's design philosophy

See `tests/summing_node_test.rs:266-322` for complete working example of the callback closure pattern.

### CallBackStream Usage

**CallBackStream is for INPUT, not output collection.**

`CallBackStream` is designed to feed test data INTO a graph at specified times. It is NOT used for collecting outputs from nodes - use the callback closure pattern instead.

Reference: https://docs.rs/wingfoil/0.1.11/wingfoil/struct.CallBackStream.html

### Interior Mutability Pattern

Wingfoil nodes must be wrapped in `RefCell` for interior mutability. This allows Wingfoil to call `cycle(&mut self)` even though nodes are stored in `Rc<dyn Node>`. The `RefCell` performs runtime borrow checking to ensure safe mutable access in the single-threaded graph execution context.

See `tests/summing_node_test.rs:280-281` for example.

Reference: https://docs.rs/wingfoil/0.1.11/wingfoil/trait.MutableNode.html

## References

- Aeron releases: https://github.com/real-logic/aeron/releases
- Testing strategy: [project.md](project.md#integration-testing-strategy)
- MediaDriverGuard implementation: See `#[cfg(test)]` modules in rusteron adapter
- Integration test examples: See `tests/` directory, especially `tests/summing_node_test.rs`
- Wingfoil documentation: https://docs.rs/wingfoil/0.1.11/wingfoil/
