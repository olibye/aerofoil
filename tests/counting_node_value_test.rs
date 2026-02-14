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
//! - Reference pattern (summing_node_test.rs): `*self.upstream.borrow().peek_ref()` (explicit deref)
//! - Value pattern (this test): `self.upstream.peek_value()` (clean, no deref)
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

mod common;

use aerofoil::nodes::AeronSubscriberValueNode;
use aerofoil::transport::rusteron::{RusteronPublisher, RusteronSubscriber};
use aerofoil::transport::AeronPublisher;
use common::{CountingNode, CountingNodeOutput, MediaDriverGuard};
use rusteron_client::IntoCString;
use std::cell::RefCell;
use std::rc::Rc;
use std::thread;
use std::time::Duration;
use wingfoil::{Graph, IntoNode, RunFor, RunMode};

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

    // Create AeronSubscriberValueNode using the builder pattern
    // Returns Rc<RefCell<...>> which can be cloned for the graph and used for upstream
    let subscriber_node = AeronSubscriberValueNode::<i64, _, RusteronSubscriber>::builder()
        .subscriber(subscriber)
        .parser(parser)
        .default(0i64)
        .build();

    // Create callback to collect outputs
    let outputs: Rc<RefCell<Vec<CountingNodeOutput>>> = Rc::new(RefCell::new(Vec::new()));
    let outputs_clone = Rc::clone(&outputs);

    let output_callback = move |output: CountingNodeOutput| {
        outputs_clone.borrow_mut().push(output);
    };

    // Create CountingNode - demonstrates value-access pattern
    let counting_node = CountingNode::new(subscriber_node.clone(), output_callback);

    // Convert to dyn Node
    let counting_graph_node = RefCell::new(counting_node).into_node();

    // Create and run Wingfoil graph
    // subscriber_node.clone() coerces to Rc<dyn Node> for the graph
    let mut graph = Graph::new(
        vec![subscriber_node, counting_graph_node],
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
