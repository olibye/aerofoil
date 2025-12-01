//! Integration test demonstrating value-based peek composition with Wingfoil and Aeron.
//!
//! This test demonstrates the **value-access pattern** using `AeronSubscriberValueNode`
//! and `StreamPeek<T>`, complementing the reference-access pattern in summing_node_test.rs.
//!
//! # Key Patterns Demonstrated
//!
//! - **Value-Access Pattern**: Using `StreamPeek<T>` for cheap-to-clone types
//! - **Clean Ergonomics**: `upstream.peek_value()` vs `*upstream.borrow().peek_ref()`
//! - **AeronSubscriberValueNode**: Value-based transport node for primitives
//! - **Type-Level Intent**: Using `StreamPeek` to signal cheap-to-clone types
//!
//! # Comparison with Reference Pattern
//!
//! Reference pattern (summing_node_test.rs):
//! ```rust,ignore
//! let value: i64 = *self.upstream.borrow().peek_ref();  // Explicit deref
//! ```
//!
//! Value pattern (this test):
//! ```rust,ignore
//! let value: i64 = self.upstream.peek_value();  // Clean, no deref
//! ```
//!
//! # Running this test
//!
//! This test requires the Aeron C libraries to be installed and uses the
//! `integration-tests` feature flag.
//!
//! Run with:
//! ```bash
//! cargo test --test counting_node_value_test --features integration-tests
//! ```

#![cfg(feature = "integration-tests")]

use aerofoil::nodes::AeronSubscriberValueNode;
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
use wingfoil::{Graph, GraphState, IntoNode, MutableNode, Node, RunFor, RunMode, StreamPeek};

/// RAII guard for managing Aeron media driver lifecycle.
struct MediaDriverGuard {
    stop_signal: Arc<AtomicBool>,
}

impl MediaDriverGuard {
    fn start() -> Result<Self, String> {
        let driver_context = AeronDriverContext::new().map_err(|e| {
            format!(
                "Failed to create media driver context: {:?}\n\n\
                 This likely means the Aeron C libraries are not installed.\n\
                 For detailed instructions, see openspec/integration-test.md",
                e
            )
        })?;

        let (stop_signal, _driver_handle) = AeronDriver::launch_embedded(driver_context, false);
        thread::sleep(Duration::from_millis(200));

        Ok(MediaDriverGuard { stop_signal })
    }
}

impl Drop for MediaDriverGuard {
    fn drop(&mut self) {
        self.stop_signal.store(true, Ordering::SeqCst);
        thread::sleep(Duration::from_millis(100));
    }
}

/// Output data from CountingNode containing the current count and last value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CountingNodeOutput {
    count: usize,
    last_value: i64,
}

/// A simple node that uses value-based access to count messages from upstream.
///
/// This demonstrates the **value-access pattern** for cheap-to-clone types:
/// - Declares upstream dependency using concrete `AeronSubscriberValueNode` type
/// - Uses `upstream.peek_value()` for clean value access (no deref needed)
/// - Implements change detection to count unique values
/// - Outputs state via callback for test observation
///
/// # Comparison with Reference Pattern
///
/// This node uses `StreamPeek<T>` which provides cleaner ergonomics for primitives:
///
/// ```rust,ignore
/// // Reference pattern (from summing_node_test.rs):
/// let value: i64 = *self.upstream.borrow().peek_ref();
///
/// // Value pattern (this node):
/// let value: i64 = self.upstream.peek_value();  // Cleaner!
/// ```
struct CountingNode<F, P, S>
where
    F: FnMut(CountingNodeOutput),
    P: FnMut(&[u8]) -> Option<i64> + 'static,
    S: aerofoil::transport::AeronSubscriber + 'static,
{
    /// Upstream node providing i64 values via peek_value()
    upstream: Rc<RefCell<AeronSubscriberValueNode<i64, P, S>>>,
    /// Count of unique values processed
    count: usize,
    /// Last value seen (for change detection)
    last_value: i64,
    /// Callback to emit output for test observation
    output_callback: F,
}

impl<F, P, S> CountingNode<F, P, S>
where
    F: FnMut(CountingNodeOutput),
    P: FnMut(&[u8]) -> Option<i64> + 'static,
    S: aerofoil::transport::AeronSubscriber + 'static,
{
    fn new(upstream: Rc<RefCell<AeronSubscriberValueNode<i64, P, S>>>, output_callback: F) -> Self {
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
    /// ```rust,ignore
    /// // Value pattern - clean and direct
    /// let current = self.upstream.peek_value();
    ///
    /// // vs. reference pattern - requires borrow + deref
    /// let current = *self.upstream.borrow().peek_ref();
    /// ```
    fn process_upstream(&mut self) {
        // Using peek_value() for clean value access - no deref needed!
        let current_value = self.upstream.peek_value();

        // Only count if the value has changed (new message arrived)
        if current_value != self.last_value || self.count == 0 {
            self.count += 1;
            self.last_value = current_value;
        }
    }
}

/// Wingfoil MutableNode implementation for CountingNode.
impl<F, P, S> MutableNode for CountingNode<F, P, S>
where
    F: FnMut(CountingNodeOutput) + 'static,
    P: FnMut(&[u8]) -> Option<i64> + 'static,
    S: aerofoil::transport::AeronSubscriber + 'static,
{
    fn cycle(&mut self, _state: &mut GraphState) -> bool {
        // Process the latest value using value-access pattern
        self.process_upstream();

        // Invoke callback with current state for test observation
        (self.output_callback)(CountingNodeOutput {
            count: self.count,
            last_value: self.last_value,
        });

        false
    }

    fn start(&mut self, state: &mut GraphState) {
        state.always_callback();
    }
}

#[test]
fn given_aeron_messages_when_value_node_processes_then_counts_correctly() {
    // Given: Start media driver with RAII guard
    let _driver = MediaDriverGuard::start()
        .expect("Failed to start media driver - see error message for installation instructions");

    // Given: Aeron context and connection
    let context = rusteron_client::AeronContext::new().expect("Failed to create Aeron context");
    let aeron = rusteron_client::Aeron::new(&context).expect("Failed to create Aeron");
    aeron.start().expect("Failed to start Aeron");

    let channel = "aeron:ipc";
    let stream_id = 1002;

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

    // Create parser function for i64 messages
    let parser = |fragment: &[u8]| -> Option<i64> {
        if fragment.len() >= 8 {
            let bytes: [u8; 8] = fragment[0..8].try_into().ok()?;
            Some(i64::from_le_bytes(bytes))
        } else {
            None
        }
    };

    // Create AeronSubscriberValueNode - value-access transport layer
    // This implements StreamPeek<i64> for clean value access
    let value_node = AeronSubscriberValueNode::new(subscriber, parser, 0i64);

    // Wrap for graph integration
    // Note: We still use RefCell for graph integration (for mutation during polling)
    // but downstream access via peek_value() is cleaner than peek_ref()
    let value_node_rc = Rc::new(RefCell::new(value_node));

    // Clone for upstream reference - CountingNode will use peek_value()
    let upstream_ref = value_node_rc.clone();

    // Cast to Rc<dyn Node> for graph
    let value_graph_node: Rc<dyn Node> = value_node_rc;

    // Create callback to collect outputs
    let outputs: Rc<RefCell<Vec<CountingNodeOutput>>> = Rc::new(RefCell::new(Vec::new()));
    let outputs_clone = Rc::clone(&outputs);

    let output_callback = move |output: CountingNodeOutput| {
        outputs_clone.borrow_mut().push(output);
    };

    // Create CountingNode - demonstrates value-access pattern
    let counting_node = CountingNode::new(upstream_ref, output_callback);

    // Convert to dyn Node
    let counting_graph_node = RefCell::new(counting_node).into_node();

    // Create and run Wingfoil graph
    let mut graph = Graph::new(
        vec![value_graph_node, counting_graph_node],
        RunMode::RealTime,
        RunFor::Cycles(10),
    );

    graph.run().expect("Graph execution failed");

    // Then: Verify the count from the collected outputs
    let collected_outputs = outputs.borrow();

    assert!(
        !collected_outputs.is_empty(),
        "Expected to collect outputs, but got none"
    );

    // Get the final output (last cycle)
    let final_output = collected_outputs.last().unwrap();

    assert_eq!(
        final_output.count, 5,
        "Expected to count 5 messages, got {}",
        final_output.count
    );

    assert_eq!(
        final_output.last_value, 5,
        "Expected last value to be 5, got {}",
        final_output.last_value
    );

    println!(
        "✓ CountingNode (value-access pattern) successfully counted {} messages, last value = {}",
        final_output.count, final_output.last_value
    );
}
