//! Integration test demonstrating the fan-out pattern with Aeron publishing.
//!
//! This test demonstrates:
//! - **Fan-Out Pattern**: Single `AeronSubscriberValueNode` feeding multiple downstream nodes
//! - **Publisher-in-Callback**: Output callbacks that capture `AeronPublisher` and publish results
//! - **Separate Output Streams**: Sum published to stream 2002, count to stream 2003
//!
//! # Architecture
//!
//! ```text
//!                           ┌─────────────┐
//!                      ┌───►│ SummingNode │───► Stream 2002 (sum)
//! Stream 2001 ───►     │    └─────────────┘
//!              Subscriber
//!                      │    ┌──────────────┐
//!                      └───►│ CountingNode │───► Stream 2003 (count)
//!                           └──────────────┘
//! ```
//!
//! # Running this test
//!
//! ```bash
//! cargo test --test sum_count_publisher_test --features integration-tests
//! ```

mod common;

use aerofoil::nodes::AeronSubscriberValueNode;
use aerofoil::transport::rusteron::{RusteronPublisher, RusteronSubscriber};
use aerofoil::transport::{AeronPublisher, AeronSubscriber, TransportError};
use common::{CountingNode, MediaDriverGuard, SummingNode};
use rusteron_client::IntoCString;
use std::cell::RefCell;
use std::rc::Rc;
use std::thread;
use std::time::Duration;
use wingfoil::{Graph, IntoNode, RunFor, RunMode};

#[test]
fn given_input_stream_when_fan_out_graph_runs_then_publishes_sum_and_count() {
    // Given: Start media driver
    let _driver = MediaDriverGuard::start()
        .expect("Failed to start media driver - see error message for installation instructions");

    let context = rusteron_client::AeronContext::new().expect("Failed to create Aeron context");
    let aeron = rusteron_client::Aeron::new(&context).expect("Failed to create Aeron");
    aeron.start().expect("Failed to start Aeron");

    let channel = "aeron:ipc";
    let input_stream = 2001;
    let sum_stream = 2002;
    let count_stream = 2003;

    // Create input publisher (stream 2001)
    let input_pub = aeron
        .async_add_publication(&channel.into_c_string(), input_stream)
        .expect("Failed to start input publication")
        .poll_blocking(Duration::from_secs(5))
        .expect("Failed to complete input publication");
    let mut input_publisher = RusteronPublisher::new(input_pub);

    // Create input subscriber (stream 2001)
    let input_sub = aeron
        .async_add_subscription(
            &channel.into_c_string(),
            input_stream,
            rusteron_client::Handlers::no_available_image_handler(),
            rusteron_client::Handlers::no_unavailable_image_handler(),
        )
        .expect("Failed to start input subscription")
        .poll_blocking(Duration::from_secs(5))
        .expect("Failed to complete input subscription");
    let input_subscriber = RusteronSubscriber::new(input_sub);

    // Create sum output publisher (stream 2002)
    let sum_pub = aeron
        .async_add_publication(&channel.into_c_string(), sum_stream)
        .expect("Failed to start sum publication")
        .poll_blocking(Duration::from_secs(5))
        .expect("Failed to complete sum publication");
    let sum_publisher = Rc::new(RefCell::new(RusteronPublisher::new(sum_pub)));

    // Create count output publisher (stream 2003)
    let count_pub = aeron
        .async_add_publication(&channel.into_c_string(), count_stream)
        .expect("Failed to start count publication")
        .poll_blocking(Duration::from_secs(5))
        .expect("Failed to complete count publication");
    let count_publisher = Rc::new(RefCell::new(RusteronPublisher::new(count_pub)));

    // Create output subscribers to verify published values
    let sum_sub = aeron
        .async_add_subscription(
            &channel.into_c_string(),
            sum_stream,
            rusteron_client::Handlers::no_available_image_handler(),
            rusteron_client::Handlers::no_unavailable_image_handler(),
        )
        .expect("Failed to start sum subscription")
        .poll_blocking(Duration::from_secs(5))
        .expect("Failed to complete sum subscription");
    let mut sum_subscriber = RusteronSubscriber::new(sum_sub);

    let count_sub = aeron
        .async_add_subscription(
            &channel.into_c_string(),
            count_stream,
            rusteron_client::Handlers::no_available_image_handler(),
            rusteron_client::Handlers::no_unavailable_image_handler(),
        )
        .expect("Failed to start count subscription")
        .poll_blocking(Duration::from_secs(5))
        .expect("Failed to complete count subscription");
    let mut count_subscriber = RusteronSubscriber::new(count_sub);

    // Wait for connections to stabilize
    thread::sleep(Duration::from_millis(200));

    // When: Publish test values [1, 2, 3, 4, 5] to input stream
    let test_values: Vec<i64> = vec![1, 2, 3, 4, 5];
    for value in &test_values {
        input_publisher
            .offer(&value.to_le_bytes())
            .unwrap_or_else(|e| panic!("Failed to publish value {}: {:?}", value, e));
    }

    // Give time for input messages to propagate
    thread::sleep(Duration::from_millis(100));

    // Create parser for i64 messages
    let parser = |fragment: &[u8]| -> Option<i64> {
        if fragment.len() >= 8 {
            Some(i64::from_le_bytes(fragment[0..8].try_into().ok()?))
        } else {
            None
        }
    };

    // Build input subscriber node (shared by both downstream nodes)
    let subscriber_node = AeronSubscriberValueNode::builder()
        .subscriber(input_subscriber)
        .parser(parser)
        .default(0i64)
        .build();

    // Create SummingNode with callback that publishes to sum stream
    let sum_pub_clone = sum_publisher.clone();
    let summing_node = SummingNode::new(subscriber_node.clone(), move |output| {
        let _ = sum_pub_clone.borrow_mut().offer(&output.sum.to_le_bytes());
    });

    // Create CountingNode with callback that publishes to count stream
    let count_pub_clone = count_publisher.clone();
    let counting_node = CountingNode::new(subscriber_node.clone(), move |output| {
        let _ = count_pub_clone
            .borrow_mut()
            .offer(&(output.count as i64).to_le_bytes());
    });

    // Build and run graph with fan-out pattern
    let mut graph = Graph::new(
        vec![
            subscriber_node,
            RefCell::new(summing_node).into_node(),
            RefCell::new(counting_node).into_node(),
        ],
        RunMode::RealTime,
        RunFor::Cycles(10),
    );

    graph.run().expect("Graph execution failed");

    // Give time for output messages to propagate
    thread::sleep(Duration::from_millis(100));

    // Then: Drain all messages from sum stream and get the final value
    let mut received_sum: Option<i64> = None;
    loop {
        let mut found = false;
        let _: Result<usize, TransportError> = sum_subscriber.poll(|fragment| {
            if fragment.len() >= 8 {
                received_sum = Some(i64::from_le_bytes(fragment[0..8].try_into().unwrap()));
                found = true;
            }
            Ok(())
        });
        if !found {
            break;
        }
    }

    // Then: Drain all messages from count stream and get the final value
    let mut received_count: Option<i64> = None;
    loop {
        let mut found = false;
        let _: Result<usize, TransportError> = count_subscriber.poll(|fragment| {
            if fragment.len() >= 8 {
                received_count = Some(i64::from_le_bytes(fragment[0..8].try_into().unwrap()));
                found = true;
            }
            Ok(())
        });
        if !found {
            break;
        }
    }

    // The final values should be sum=15 and count=5
    assert!(
        received_sum.is_some(),
        "Expected to receive sum on stream 2002"
    );
    assert!(
        received_count.is_some(),
        "Expected to receive count on stream 2003"
    );

    // Check final values
    assert_eq!(
        received_sum.unwrap(),
        15,
        "Expected sum of 15 (1+2+3+4+5), got {}",
        received_sum.unwrap()
    );
    assert_eq!(
        received_count.unwrap(),
        5,
        "Expected count of 5, got {}",
        received_count.unwrap()
    );

    println!("✓ Fan-out pattern test passed: sum={}, count={}", 15, 5);
}
