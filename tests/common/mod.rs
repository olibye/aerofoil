#![allow(dead_code)]
//! Common test utilities for integration tests.
//!
//! This module provides shared helpers used across integration tests,
//! following the project convention to "Factor out common code into helper
//! functions and classes even for tests".
//!
//! # Test Nodes
//!
//! This module provides reusable test nodes that demonstrate Wingfoil patterns:
//! - [`SummingNode`] - Demonstrates reference-based access with `StreamPeekRef`
//! - [`CountingNode`] - Demonstrates value-based access with `StreamPeek`

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use wingfoil::{GraphState, MutableNode, StreamPeek, StreamPeekRef, UpStreams};

/// RAII guard for managing Aeron media driver lifecycle.
///
/// The media driver is automatically started on creation and stopped on drop,
/// ensuring proper cleanup even if the test panics.
///
/// # Example
///
/// See `tests/summing_node_test.rs` for usage in a complete integration test.
pub struct MediaDriverGuard {
    stop_signal: Arc<AtomicBool>,
}

impl MediaDriverGuard {
    /// Starts an embedded Aeron media driver.
    ///
    /// # Errors
    ///
    /// Returns an error if the media driver cannot be started, with a helpful
    /// message pointing to installation instructions.
    ///
    /// # Example
    ///
    /// See `tests/summing_node_test.rs` for usage in a complete integration test.
    #[cfg(feature = "embedded-driver")]
    pub fn start() -> Result<Self, String> {
        use rusteron_media_driver::{AeronDriver, AeronDriverContext};

        // Try to create driver context - this will fail if libaeron_driver.dylib is not available
        let driver_context = AeronDriverContext::new().map_err(|e| {
            format!(
                "Failed to create media driver context: {:?}\n\n\
                 This likely means the Aeron C libraries are not installed.\n\n\
                 On macOS, you need to:\n\
                 1. Download Aeron from https://github.com/real-logic/aeron/releases\n\
                 2. Extract and ensure libaeron_driver.dylib is in a library path\n\
                 3. Or set DYLD_LIBRARY_PATH to point to the lib directory\n\n\
                 For detailed instructions, see docs/development-guide.md",
                e
            )
        })?;

        // Launch embedded driver - returns (Arc<AtomicBool>, JoinHandle)
        let (stop_signal, _driver_handle) = AeronDriver::launch_embedded(driver_context, false);

        // Give the driver time to initialize
        thread::sleep(Duration::from_millis(200));

        Ok(MediaDriverGuard { stop_signal })
    }

    #[cfg(not(feature = "embedded-driver"))]
    pub fn start() -> Result<Self, String> {
        Err("Embedded driver feature not enabled. Enable 'embedded-driver' feature.".to_string())
    }
}

impl Drop for MediaDriverGuard {
    fn drop(&mut self) {
        // Signal the driver to stop
        self.stop_signal.store(true, Ordering::SeqCst);
        // Give it time to shut down cleanly
        thread::sleep(Duration::from_millis(100));
    }
}

/// Output data from SummingNode containing the current sum and message count.
///
/// This struct is emitted by the node to communicate its state, following
/// Wingfoil's single-threaded pattern without thread-safe primitives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SummingNodeOutput {
    pub sum: i64,
    pub count: usize,
}

/// A simple node that reads from an upstream source and maintains a running sum.
///
/// This demonstrates the **reference-based access pattern** for Wingfoil:
/// - Declares upstream dependency as `Rc<RefCell<T>>` where `T: StreamPeekRef<i64>`
/// - Uses `upstream.borrow().peek_ref()` to access latest value
/// - Implements change detection to identify new values
/// - Maintains running sum of all received values
/// - Outputs state via callback for test observation
///
/// This node is **transport-agnostic** - it works with any `StreamPeekRef<i64>` implementation,
/// making it easy to test with mock streams and compose with different data sources.
///
/// # Example
///
/// See `examples/summing_node_composition.rs` for a complete example.
pub struct SummingNode<T, F>
where
    T: StreamPeekRef<i64>,
    F: FnMut(SummingNodeOutput),
{
    /// Upstream node providing i64 values via peek_ref()
    upstream: Rc<RefCell<T>>,
    /// Running sum of all values seen
    running_sum: i64,
    /// Count of unique values processed
    message_count: usize,
    /// Last value seen (for change detection)
    last_value: i64,
    /// Callback to emit output for test observation
    output_callback: F,
}

impl<T, F> SummingNode<T, F>
where
    T: StreamPeekRef<i64>,
    F: FnMut(SummingNodeOutput),
{
    /// Creates a new `SummingNode` that processes values from the upstream node.
    ///
    /// # Parameters
    ///
    /// - `upstream`: Reference-counted upstream node implementing `StreamPeekRef<i64>`
    /// - `output_callback`: Callback invoked on each cycle with current state
    pub fn new(upstream: Rc<RefCell<T>>, output_callback: F) -> Self {
        SummingNode {
            upstream,
            running_sum: 0,
            message_count: 0,
            last_value: 0,
            output_callback,
        }
    }

    /// Reads the latest value from upstream and updates the running sum.
    ///
    /// This method demonstrates the core pattern for stateful processing with Wingfoil streams:
    /// - Use `upstream.borrow().peek_ref()` to access the latest value from the upstream node
    /// - Track the last processed value to detect when new values arrive
    /// - Update internal state based on new values
    fn process_upstream(&mut self) {
        // Borrow the upstream node and peek at the latest value
        let current_value = *self.upstream.borrow().peek_ref();

        // Only process if the value has changed (new message arrived)
        if current_value != self.last_value || self.message_count == 0 {
            self.running_sum += current_value;
            self.message_count += 1;
            self.last_value = current_value;
        }
    }
}

impl<T, F> MutableNode for SummingNode<T, F>
where
    T: StreamPeekRef<i64> + 'static,
    F: FnMut(SummingNodeOutput) + 'static,
{
    fn cycle(&mut self, _state: &mut GraphState) -> anyhow::Result<bool> {
        // Process the latest value from upstream using peek pattern
        self.process_upstream();

        // Invoke callback with current state for test observation
        (self.output_callback)(SummingNodeOutput {
            sum: self.running_sum,
            count: self.message_count,
        });

        // Return false to indicate we want to continue processing
        Ok(false)
    }

    fn start(&mut self, state: &mut GraphState) -> anyhow::Result<()> {
        state.always_callback();
        Ok(())
    }

    fn upstreams(&self) -> UpStreams {
        UpStreams::none()
    }
}

/// Output data from CountingNode containing the current count and last value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CountingNodeOutput {
    pub count: usize,
    pub last_value: i64,
}

/// A simple node that uses value-based access to count messages from upstream.
///
/// This demonstrates the **value-access pattern** for cheap-to-clone types:
/// - Declares upstream dependency using `Rc<RefCell<T>>` where `T: StreamPeekRef<i64>`
/// - Uses `upstream.peek_value()` which is auto-implemented by Wingfoil for `RefCell<T>`
/// - Implements change detection to count unique values
/// - Outputs state via callback for test observation
///
/// # Comparison with Reference Pattern
///
/// Reference pattern (SummingNode): `*self.upstream.borrow().peek_ref()`
/// Value pattern (CountingNode): `self.upstream.peek_value()` (cleaner!)
///
/// # How Value Access Works
///
/// Wingfoil auto-implements `StreamPeek<T>` for `RefCell<STREAM>` where `STREAM: StreamPeekRef<T>`.
/// This means when we have `Rc<RefCell<AeronSubscriberValueNode>>`, the `RefCell` layer provides
/// the `peek_value()` method automatically.
///
/// # Example
///
/// See `examples/counting_node_composition.rs` for a complete example.
pub struct CountingNode<T, F>
where
    T: StreamPeekRef<i64>,
    F: FnMut(CountingNodeOutput),
{
    /// Upstream node providing i64 values via peek_value()
    /// Note: Uses StreamPeekRef; Wingfoil auto-implements StreamPeek for RefCell<T>
    upstream: Rc<RefCell<T>>,
    /// Count of unique values processed
    count: usize,
    /// Last value seen (for change detection)
    last_value: i64,
    /// Callback to emit output for test observation
    output_callback: F,
}

impl<T, F> CountingNode<T, F>
where
    T: StreamPeekRef<i64>,
    F: FnMut(CountingNodeOutput),
{
    /// Creates a new `CountingNode` that counts values from the upstream node.
    ///
    /// # Parameters
    ///
    /// - `upstream`: Reference-counted upstream node implementing `StreamPeekRef<i64>`
    /// - `output_callback`: Callback invoked on each cycle with current state
    pub fn new(upstream: Rc<RefCell<T>>, output_callback: F) -> Self {
        CountingNode {
            upstream,
            count: 0,
            last_value: 0,
            output_callback,
        }
    }

    /// Reads the latest value from upstream using the value-access pattern.
    ///
    /// **Key difference from reference pattern:**
    /// - Value pattern: `self.upstream.peek_value()` (clean and direct)
    /// - Reference pattern: `*self.upstream.borrow().peek_ref()` (requires borrow + deref)
    fn process_upstream(&mut self) {
        // Using peek_value() for clean value access - no deref needed!
        // Wingfoil auto-implements StreamPeek::peek_value() for RefCell<T> where T: StreamPeekRef
        let current_value = self.upstream.peek_value();

        // Only count if the value has changed (new message arrived)
        if current_value != self.last_value || self.count == 0 {
            self.count += 1;
            self.last_value = current_value;
        }
    }
}

impl<T, F> MutableNode for CountingNode<T, F>
where
    T: StreamPeekRef<i64> + 'static,
    F: FnMut(CountingNodeOutput) + 'static,
{
    fn cycle(&mut self, _state: &mut GraphState) -> anyhow::Result<bool> {
        // Process the latest value using value-access pattern
        self.process_upstream();

        // Invoke callback with current state for test observation
        (self.output_callback)(CountingNodeOutput {
            count: self.count,
            last_value: self.last_value,
        });

        Ok(false)
    }

    fn start(&mut self, state: &mut GraphState) -> anyhow::Result<()> {
        state.always_callback();
        Ok(())
    }

    fn upstreams(&self) -> wingfoil::UpStreams {
        UpStreams::none()
    }
}
