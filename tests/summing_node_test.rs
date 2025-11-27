//! Integration test demonstrating stateful stream processing with Wingfoil and Aeron.
//!
//! This test creates a SummingNode that implements Wingfoil's MutableNode trait,
//! polls a Rusteron subscriber for i64 values, and maintains a running sum.
//! It demonstrates the complete pattern for building stateful Wingfoil nodes
//! that process Aeron messages in HFT systems.
//!
//! # Key Patterns Demonstrated
//!
//! - **Wingfoil Node**: Implementing `MutableNode` trait for graph-based execution
//! - **Aeron Transport**: Using RusteronSubscriber for zero-copy message receipt
//! - **Shared State**: Using Arc<Mutex<>> to access node state after graph execution
//! - **Non-blocking Poll**: Subscriber poll returns immediately if no messages available
//! - **Lifecycle Management**: RAII guards for media driver and proper cleanup
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

use aerofoil::transport::rusteron::{RusteronPublisher, RusteronSubscriber};
use aerofoil::transport::{AeronPublisher, AeronSubscriber};
use rusteron_client::IntoCString;
use rusteron_media_driver::{AeronDriverContext, AeronDriver};
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use wingfoil::{Graph, GraphState, IntoNode, MutableNode, RunFor, RunMode};

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
        let driver_context = AeronDriverContext::new()
            .map_err(|e| format!(
                "Failed to create media driver context: {:?}\n\n\
                 This likely means the Aeron C libraries are not installed.\n\n\
                 On macOS, you need to:\n\
                 1. Download Aeron from https://github.com/real-logic/aeron/releases\n\
                 2. Extract and ensure libaeron_driver.dylib is in a library path\n\
                 3. Or set DYLD_LIBRARY_PATH to point to the lib directory\n\n\
                 For detailed instructions, see openspec/integration-test.md",
                e
            ))?;

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

/// Shared state for SummingNode that can be accessed from outside the graph.
///
/// This struct holds the computed results in thread-safe containers
/// so they can be verified after graph execution completes.
#[derive(Clone)]
struct SummingNodeState {
    sum: Arc<Mutex<i64>>,
    count: Arc<Mutex<usize>>,
}

impl SummingNodeState {
    fn new() -> Self {
        SummingNodeState {
            sum: Arc::new(Mutex::new(0)),
            count: Arc::new(Mutex::new(0)),
        }
    }

    fn get_sum(&self) -> i64 {
        *self.sum.lock().unwrap()
    }

    fn get_count(&self) -> usize {
        *self.count.lock().unwrap()
    }
}

/// A simple node that polls a Rusteron subscriber and maintains a running sum.
///
/// This demonstrates the pattern for stateful stream processing:
/// - Wraps a transport subscriber
/// - Maintains processing state (running sum)
/// - Polls for input in cycle() method (called by Wingfoil)
/// - Shares state via Arc<Mutex<>> for verification
struct SummingNode {
    subscriber: RusteronSubscriber,
    state: SummingNodeState,
}

impl SummingNode {
    fn new(subscriber: RusteronSubscriber, state: SummingNodeState) -> Self {
        SummingNode {
            subscriber,
            state,
        }
    }

    /// Polls the subscriber and processes received messages (non-blocking).
    ///
    /// This method demonstrates the core pattern for stateful processing:
    /// - Poll subscriber (returns immediately if no messages)
    /// - Parse binary data from fragment buffers (zero-copy)
    /// - Update internal state based on received values
    fn poll_and_process(&mut self) -> Result<usize, aerofoil::transport::TransportError> {
        self.subscriber.poll(|fragment| {
            // Parse i64 from fragment buffer (little-endian, 8 bytes)
            if fragment.len() >= 8 {
                let bytes: [u8; 8] = fragment[0..8].try_into().unwrap();
                let value = i64::from_le_bytes(bytes);

                // Update running sum in shared state
                *self.state.sum.lock().unwrap() += value;
                *self.state.count.lock().unwrap() += 1;
            }
            Ok(())
        })
    }
}

/// Wingfoil MutableNode implementation for SummingNode.
///
/// This enables the node to be registered in a Wingfoil graph and receive
/// automatic cycle callbacks for polling and processing messages.
impl MutableNode for SummingNode {
    /// Called by Wingfoil on each graph cycle to poll for and process messages.
    ///
    /// The node polls the Aeron subscriber (non-blocking) and updates its
    /// running sum for any received i64 values. Returns false to indicate
    /// the node should continue processing (never completes on its own).
    fn cycle(&mut self, _state: &mut GraphState) -> bool {
        // Poll and process any available messages
        let _ = self.poll_and_process();

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
fn test_summing_node_integration() {
    // Given: Start media driver with RAII guard (auto cleanup on drop)
    let _driver = MediaDriverGuard::start()
        .expect("Failed to start media driver - see error message for installation instructions");

    // Given: Aeron context and connection
    let context = rusteron_client::AeronContext::new().expect("Failed to create Aeron context");

    let aeron = rusteron_client::Aeron::new(&context).expect("Failed to create Aeron");
    aeron.start().expect("Failed to start Aeron");

    let channel = "aeron:ipc";
    let stream_id = 1001;

    // Create publisher asynchronously
    let async_pub = aeron
        .async_add_publication(&channel.into_c_string(), stream_id)
        .expect("Failed to start async publication");

    let publication = async_pub
        .poll_blocking(Duration::from_secs(5))
        .expect("Failed to complete publication");

    let mut publisher = RusteronPublisher::new(publication);

    // Create subscriber asynchronously
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

    // Create shared state for verificationafter graph execution
    let state = SummingNodeState::new();
    let verification_state = state.clone();

    // Create SummingNode wrapping the subscriber and shared state
    let summing_node = SummingNode::new(subscriber, state);

    // Wrap in RefCell for Wingfoil's interior mutability pattern
    let node = RefCell::new(summing_node).into_node();

    // Create and run Wingfoil graph with the SummingNode
    // Run for 10 cycles to poll and process messages
    let mut graph = Graph::new(
        vec![node],
        RunMode::RealTime,
        RunFor::Cycles(10),
    );

    graph.run().expect("Graph execution failed");

    // Then: Verify the sum and message count using the shared state
    assert_eq!(
        verification_state.get_count(),
        5,
        "Expected to receive 5 messages, got {}",
        verification_state.get_count()
    );

    assert_eq!(
        verification_state.get_sum(),
        15,
        "Expected sum of 15 (1+2+3+4+5), got {}",
        verification_state.get_sum()
    );

    println!("✓ SummingNode successfully processed {} messages with sum = {}",
             verification_state.get_count(), verification_state.get_sum());
}
