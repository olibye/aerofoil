//! Integration test demonstrating stateful stream processing with Rusteron.
//!
//! This test creates a SummingNode that polls a Rusteron subscriber for i64 values
//! and maintains a running sum. It demonstrates the core pattern for building
//! stateful processors in HFT systems.
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
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

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

/// A simple node that polls a Rusteron subscriber and maintains a running sum.
///
/// This demonstrates the pattern for stateful stream processing:
/// - Wraps a transport subscriber
/// - Maintains processing state (running sum)
/// - Polls for input in poll_and_process() method
/// - Provides output accessor for verification
struct SummingNode {
    subscriber: RusteronSubscriber,
    running_sum: i64,
    message_count: usize,
}

impl SummingNode {
    fn new(subscriber: RusteronSubscriber) -> Self {
        SummingNode {
            subscriber,
            running_sum: 0,
            message_count: 0,
        }
    }

    fn sum(&self) -> i64 {
        self.running_sum
    }

    fn count(&self) -> usize {
        self.message_count
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

                // Update running sum
                self.running_sum += value;
                self.message_count += 1;
            }
            Ok(())
        })
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

    // Create SummingNode wrapping the subscriber
    let mut summing_node = SummingNode::new(subscriber);

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

    // Poll and process messages in a loop (simulating cycle callbacks)
    for _ in 0..10 {
        let _ = summing_node.poll_and_process();
        thread::sleep(Duration::from_millis(10));
    }

    // Then: Verify the sum and message count
    assert_eq!(
        summing_node.count(),
        5,
        "Expected to receive 5 messages, got {}",
        summing_node.count()
    );

    assert_eq!(
        summing_node.sum(),
        15,
        "Expected sum of 15 (1+2+3+4+5), got {}",
        summing_node.sum()
    );

    println!("✓ SummingNode successfully processed {} messages with sum = {}",
             summing_node.count(), summing_node.sum());
}
