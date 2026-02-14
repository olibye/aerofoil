//! Integration test demonstrating peek-based node composition with Wingfoil and Aeron.
//!
//! This test demonstrates the proper Wingfoil node composition pattern:
//! - **AeronSubscriberValueRefNode**: Transport layer that polls Aeron and implements StreamPeekRef<i64>
//! - **SummingNode**: Business logic layer that uses peek() to access upstream values
//! - **Dual-Rc Pattern**: Sharing nodes between graph and upstream references
//!
//! # Key Patterns Demonstrated
//!
//! - **Separation of Concerns**: Transport (AeronSubscriberValueRefNode) separate from logic (SummingNode)
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

mod common;

use aerofoil::nodes::AeronSubscriberValueRefNode;
use aerofoil::transport::rusteron::{RusteronPublisher, RusteronSubscriber};
use aerofoil::transport::AeronPublisher;
use common::{MediaDriverGuard, SummingNode, SummingNodeOutput};
use rusteron_client::IntoCString;
use std::cell::RefCell;
use std::rc::Rc;
use std::thread;
use std::time::Duration;
use wingfoil::{Graph, IntoNode, RunFor, RunMode};

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

    // Create AeronSubscriberValueRefNode using the builder pattern
    // Returns Rc<RefCell<...>> which can be cloned for the graph and used for upstream
    let subscriber_node = AeronSubscriberValueRefNode::<i64, _, RusteronSubscriber>::builder()
        .subscriber(subscriber)
        .parser(parser)
        .default(0i64)
        .build_ref();

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
    // to access values from the upstream AeronSubscriberValueRefNode
    let summing_node = SummingNode::new(subscriber_node.clone(), output_callback);

    // Convert SummingNode to dyn Node using into_node() helper
    let summing_graph_node = RefCell::new(summing_node).into_node();

    // Create and run Wingfoil graph with both nodes:
    // - AeronSubscriberValueRefNode: Polls Aeron and provides values via peek_ref()
    // - SummingNode: Consumes values via peek_ref() and maintains running sum
    // Run for 10 cycles to poll and process messages
    // subscriber_node coerces to Rc<dyn Node> for the graph
    let mut graph = Graph::new(
        vec![subscriber_node, summing_graph_node],
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
