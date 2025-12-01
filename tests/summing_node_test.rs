//! Integration test demonstrating peek-based node composition with Wingfoil and Aeron.
//!
//! This test demonstrates the proper Wingfoil node composition pattern:
//! - **AeronSubscriberNode**: Transport layer that polls Aeron and implements StreamPeekRef<i64>
//! - **SummingNode**: Business logic layer that uses peek() to access upstream values
//! - **Dual-Rc Pattern**: Sharing nodes between graph and upstream references
//!
//! # Key Patterns Demonstrated
//!
//! - **Separation of Concerns**: Transport (AeronSubscriberNode) separate from logic (SummingNode)
//! - **Peek-Based Composition**: Downstream nodes use `peek_ref()` to access upstream values
//! - **Element Types**: Using Wingfoil's Element trait (Debug + Clone + Default + 'static)
//! - **Dual-Rc Pattern**: Manual Rc<RefCell<>> management for graph integration
//! - **Callback Output**: Using closures to observe node state (for testing only)
//! - **Lifecycle Management**: RAII guards for media driver and proper cleanup
//!
//! # Wingfoil Single-threaded Pattern
//!
//! Following Wingfoil's design, nodes execute in a single-threaded context:
//! - Nodes maintain simple, non-synchronized state
//! - Upstream dependencies use `Rc<RefCell<T>>` (single-threaded reference counting)
//! - Output via callback closure that captures Rc<RefCell<Vec<T>>> for test observation
//!
//! Reference: https://docs.rs/wingfoil/0.1.11/wingfoil/
//!
//! # Running this test
//!
//! This test requires the Aeron C libraries to be installed and uses the
//! `integration-tests` feature flag.
//!
//! Run with:
//! ```bash
//! cargo test --test summing_node_test --features integration-tests
//! ```
//!
//! If you don't have Aeron installed, the test will be skipped (not compiled).
//! For installation instructions, see openspec/integration-test.md

#![cfg(feature = "integration-tests")]

use aerofoil::nodes::AeronSubscriberNode;
use aerofoil::transport::rusteron::{RusteronPublisher, RusteronSubscriber};
use aerofoil::transport::AeronPublisher;
use rusteron_client::IntoCString;
use rusteron_media_driver::{AeronDriver, AeronDriverContext};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use wingfoil::{Graph, GraphState, IntoNode, MutableNode, Node, RunFor, RunMode, StreamPeekRef};

/// RAII guard for managing Aeron media driver lifecycle.
///
/// The media driver is automatically started on creation and stopped on drop,
/// ensuring proper cleanup even if the test panics.
struct MediaDriverGuard {
    stop_signal: Arc<AtomicBool>,
}

impl MediaDriverGuard {
    /// Starts an embedded Aeron media driver.
    ///
    /// # Errors
    ///
    /// Returns an error if the media driver cannot be started, with a helpful
    /// message pointing to installation instructions.
    fn start() -> Result<Self, String> {
        // Try to create driver context - this will fail if libaeron_driver.dylib is not available
        let driver_context = AeronDriverContext::new().map_err(|e| {
            format!(
                "Failed to create media driver context: {:?}\n\n\
                 This likely means the Aeron C libraries are not installed.\n\n\
                 On macOS, you need to:\n\
                 1. Download Aeron from https://github.com/real-logic/aeron/releases\n\
                 2. Extract and ensure libaeron_driver.dylib is in a library path\n\
                 3. Or set DYLD_LIBRARY_PATH to point to the lib directory\n\n\
                 For detailed instructions, see openspec/integration-test.md",
                e
            )
        })?;

        // Launch embedded driver - returns (Arc<AtomicBool>, JoinHandle)
        let (stop_signal, _driver_handle) = AeronDriver::launch_embedded(driver_context, false);

        // Give the driver time to initialize
        thread::sleep(Duration::from_millis(200));

        Ok(MediaDriverGuard { stop_signal })
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
struct SummingNodeOutput {
    sum: i64,
    count: usize,
}

/// A simple node that reads from an upstream Stream<i64> and maintains a running sum.
///
/// This demonstrates the peek-based pattern for stateful stream processing with Wingfoil:
/// - Declares upstream dependency as `Rc<RefCell<T>>` where `T: StreamPeekRef<i64>`
/// - Uses `upstream.borrow().peek_ref()` to access latest value
/// - Implements change detection to identify new values
/// - Maintains running sum of all received values
/// - Outputs state via callback for test observation
///
/// This node is **transport-agnostic** - it works with any Stream<i64> implementation,
/// making it easy to test with mock streams and compose with different data sources.
///
/// For testing, use a closure that captures `Rc<RefCell<Vec<T>>>` to collect outputs.
/// This is Wingfoil's single-threaded pattern for observing node state during tests.
///
/// Reference: https://docs.rs/wingfoil/0.1.11/wingfoil/trait.MutableNode.html
struct SummingNode<T, F>
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
    fn new(upstream: Rc<RefCell<T>>, output_callback: F) -> Self {
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
    ///
    /// This is transport-agnostic - the upstream could be an AeronSubscriberNode,
    /// a mock stream for testing, or any other Stream<i64> implementation.
    fn process_upstream(&mut self) {
        // Borrow the upstream node and peek at the latest value
        let current_value = *self.upstream.borrow().peek_ref();

        // Only process if the value has changed (new message arrived)
        // Note: This is a simple change detection. For production use cases,
        // you might want to use message sequence numbers or timestamps.
        if current_value != self.last_value || self.message_count == 0 {
            self.running_sum += current_value;
            self.message_count += 1;
            self.last_value = current_value;
        }
    }
}

/// Wingfoil MutableNode implementation for SummingNode.
///
/// This enables the node to be registered in a Wingfoil graph and receive
/// automatic cycle callbacks for polling and processing messages.
///
/// Reference: https://docs.rs/wingfoil/0.1.11/wingfoil/trait.MutableNode.html
impl<T, F> MutableNode for SummingNode<T, F>
where
    T: StreamPeekRef<i64> + 'static,
    F: FnMut(SummingNodeOutput) + 'static,
{
    /// Called by Wingfoil on each graph cycle to process upstream values.
    ///
    /// The node checks for new values from the upstream node via peek_ref()
    /// and updates its running sum. After processing, it invokes the output
    /// callback with the current state for test observation.
    /// Returns false to indicate the node should continue processing.
    fn cycle(&mut self, _state: &mut GraphState) -> bool {
        // Process the latest value from upstream using peek pattern
        self.process_upstream();

        // Invoke callback with current state for test observation
        (self.output_callback)(SummingNodeOutput {
            sum: self.running_sum,
            count: self.message_count,
        });

        // Return false to indicate we want to continue processing
        // (the graph will control when to stop based on its run configuration)
        false
    }

    /// Register this node to be called on every cycle.
    ///
    /// This ensures the node continuously polls for incoming messages
    /// throughout the graph's execution.
    fn start(&mut self, state: &mut GraphState) {
        state.always_callback();
    }
}

#[test]
fn given_aeron_messages_when_summing_node_processes_then_calculates_correct_sum() {
    // Given: Start media driver with RAII guard (auto cleanup on drop)
    let _driver = MediaDriverGuard::start()
        .expect("Failed to start media driver - see error message for installation instructions");

    // Given: Aeron context and connection
    let context = rusteron_client::AeronContext::new().expect("Failed to create Aeron context");

    let aeron = rusteron_client::Aeron::new(&context).expect("Failed to create Aeron");
    aeron.start().expect("Failed to start Aeron");

    let channel = "aeron:ipc";
    let stream_id = 1001;

    // Create publisher
    let async_pub = aeron
        .async_add_publication(&channel.into_c_string(), stream_id)
        .expect("Failed to start async publication");

    let publication = async_pub
        .poll_blocking(Duration::from_secs(5))
        .expect("Failed to complete publication");

    let mut publisher = RusteronPublisher::new(publication);

    // Create subscriber
    let async_sub = aeron
        .async_add_subscription(
            &channel.into_c_string(),
            stream_id,
            rusteron_client::Handlers::no_available_image_handler(),
            rusteron_client::Handlers::no_unavailable_image_handler(),
        )
        .expect("Failed to start async subscription");

    let subscription = async_sub
        .poll_blocking(Duration::from_secs(5))
        .expect("Failed to complete subscription");

    let subscriber = RusteronSubscriber::new(subscription);

    // Wait for connection to stabilize
    thread::sleep(Duration::from_millis(200));

    // When: Publish sequence of i64 values (1, 2, 3, 4, 5)
    let test_values: Vec<i64> = vec![1, 2, 3, 4, 5];

    for value in &test_values {
        let bytes = value.to_le_bytes();
        publisher
            .offer(&bytes)
            .unwrap_or_else(|e| panic!("Failed to publish value {}: {:?}", value, e));
    }

    // Give time for messages to propagate
    thread::sleep(Duration::from_millis(100));

    // Create parser function for i64 messages (little-endian, 8 bytes)
    let parser = |fragment: &[u8]| -> Option<i64> {
        if fragment.len() >= 8 {
            let bytes: [u8; 8] = fragment[0..8].try_into().ok()?;
            Some(i64::from_le_bytes(bytes))
        } else {
            None
        }
    };

    // Create AeronSubscriberNode - this is the transport layer that polls Aeron
    // and implements StreamPeekRef<i64> for downstream consumption
    let aeron_node = AeronSubscriberNode::new(subscriber, parser, 0i64);

    // DUAL-RC PATTERN: We need to share this node in two ways:
    // 1. As upstream reference (concrete type) for SummingNode to call peek_ref()
    // 2. As graph node (Rc<dyn Node>) for the graph's heterogeneous vector
    //
    // We manually create Rc<RefCell<>> instead of using into_node() because:
    // - into_node() consumes the value and returns Rc<dyn Node> (type-erased)
    // - We need to clone BEFORE type erasure to preserve concrete type for peek access
    let aeron_node_rc: Rc<RefCell<_>> = Rc::new(RefCell::new(aeron_node));

    // Clone the Rc for upstream reference - keeps concrete type for peek_ref()
    let upstream_ref = aeron_node_rc.clone();

    // Cast to Rc<dyn Node> for graph - type erasure to fit in heterogeneous vector
    // We can upcast Rc<RefCell<ConcreteNode>> to Rc<dyn Node> because RefCell<T> implements Node
    let aeron_graph_node: Rc<dyn Node> = aeron_node_rc;

    // Create a callback to collect outputs from the SummingNode
    // This uses Rc<RefCell<>> which is Wingfoil's single-threaded pattern for test observation
    // Reference: https://docs.rs/wingfoil/0.1.11/wingfoil/
    let outputs: Rc<RefCell<Vec<SummingNodeOutput>>> = Rc::new(RefCell::new(Vec::new()));
    let outputs_clone = Rc::clone(&outputs);

    // Create callback closure that captures the outputs vector
    let output_callback = move |output: SummingNodeOutput| {
        outputs_clone.borrow_mut().push(output);
    };

    // Create SummingNode - this is the business logic layer that uses peek_ref()
    // to access values from the upstream AeronSubscriberNode
    let summing_node = SummingNode::new(upstream_ref, output_callback);

    // Convert SummingNode to dyn Node using into_node() helper
    let summing_graph_node = RefCell::new(summing_node).into_node();

    // Create and run Wingfoil graph with both nodes:
    // - AeronSubscriberNode: Polls Aeron and provides values via peek_ref()
    // - SummingNode: Consumes values via peek_ref() and maintains running sum
    // Run for 10 cycles to poll and process messages
    let mut graph = Graph::new(
        vec![aeron_graph_node, summing_graph_node],
        RunMode::RealTime,
        RunFor::Cycles(10),
    );

    graph.run().expect("Graph execution failed");

    // Then: Verify the sum and message count from the collected outputs
    let collected_outputs = outputs.borrow();

    // We should have output from each cycle (10 cycles)
    assert!(
        !collected_outputs.is_empty(),
        "Expected to collect outputs, but got none"
    );

    // Get the final output (last cycle)
    let final_output = collected_outputs.last().unwrap();

    assert_eq!(
        final_output.count, 5,
        "Expected to receive 5 messages, got {}",
        final_output.count
    );

    assert_eq!(
        final_output.sum, 15,
        "Expected sum of 15 (1+2+3+4+5), got {}",
        final_output.sum
    );

    println!(
        "✓ SummingNode successfully processed {} messages with sum = {}",
        final_output.count, final_output.sum
    );
}
