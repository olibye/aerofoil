//! Integration test demonstrating the fan-out pattern with Aeron publishing.
//! Runs with both rusteron and aeron-rs backends if enabled.

#![cfg(any(feature = "rusteron", feature = "aeron-rs"))]

mod common;

use aerofoil::nodes::AeronSubscriberValueNode;
use aerofoil::transport::{AeronPublisher, AeronSubscriber, TransportError};
use common::{CountingNode, MediaDriverGuard, SummingNode};
use std::cell::RefCell;
use std::rc::Rc;
use std::thread;
use std::time::Duration;
use wingfoil::{Graph, IntoNode, RunFor, RunMode};

/// Generic test runner that works with any AeronPublisher/AeronSubscriber implementation
fn run_fan_out_test<P, S>(
    mut input_publisher: P,
    input_subscriber: S,
    sum_publisher: P,
    mut sum_subscriber: S,
    count_publisher: P,
    mut count_subscriber: S,
) where
    P: AeronPublisher + 'static,
    S: AeronSubscriber + 'static,
{
    // Wrappers for publishers to be used in callbacks
    let sum_publisher_rc = Rc::new(RefCell::new(sum_publisher));
    let count_publisher_rc = Rc::new(RefCell::new(count_publisher));

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
    let sum_pub_clone = sum_publisher_rc.clone();
    let summing_node = SummingNode::new(subscriber_node.clone(), move |output| {
        let _ = sum_pub_clone.borrow_mut().offer(&output.sum.to_le_bytes());
    });

    // Create CountingNode with callback that publishes to count stream
    let count_pub_clone = count_publisher_rc.clone();
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
}

#[cfg(feature = "rusteron")]
mod rusteron_test {
    use super::*;
    use aerofoil::transport::rusteron::{RusteronPublisher, RusteronSubscriber};
    use rusteron_client::IntoCString;

    #[test]
    fn given_input_stream_when_fan_out_graph_runs_with_rusteron_then_publishes_sum_and_count() {
        let _driver = MediaDriverGuard::start()
            .expect("Failed to start media driver - see error message for installation instructions");

        let context = rusteron_client::AeronContext::new().expect("Failed to create Aeron context");
        let aeron = rusteron_client::Aeron::new(&context).expect("Failed to create Aeron");
        aeron.start().expect("Failed to start Aeron");

        let channel = "aeron:ipc";
        let input_stream = 2001;
        let sum_stream = 2002;
        let count_stream = 2003;

        // Helper to creating blocking publications
        let create_pub = |stream_id| {
            aeron
                .async_add_publication(&channel.into_c_string(), stream_id)
                .expect("Failed to start publication")
                .poll_blocking(Duration::from_secs(5))
                .expect("Failed to complete publication")
        };

        // Helper to creating blocking subscriptions
        let create_sub = |stream_id| {
            aeron
                .async_add_subscription(
                    &channel.into_c_string(),
                    stream_id,
                    rusteron_client::Handlers::no_available_image_handler(),
                    rusteron_client::Handlers::no_unavailable_image_handler(),
                )
                .expect("Failed to start subscription")
                .poll_blocking(Duration::from_secs(5))
                .expect("Failed to complete subscription")
        };

        let input_publisher = RusteronPublisher::new(create_pub(input_stream));
        let input_subscriber = RusteronSubscriber::new(create_sub(input_stream));
        let sum_publisher = RusteronPublisher::new(create_pub(sum_stream));
        let sum_subscriber = RusteronSubscriber::new(create_sub(sum_stream));
        let count_publisher = RusteronPublisher::new(create_pub(count_stream));
        let count_subscriber = RusteronSubscriber::new(create_sub(count_stream));

        run_fan_out_test(
            input_publisher,
            input_subscriber,
            sum_publisher,
            sum_subscriber,
            count_publisher,
            count_subscriber,
        );
    }
}

#[cfg(feature = "aeron-rs")]
mod aeron_rs_test {
    use super::*;
    use aerofoil::transport::aeron_rs::{AeronRsPublisher, AeronRsSubscriber};
    use aeron_rs::aeron::Aeron;
    use aeron_rs::context::Context;
    use std::ffi::CString;
    use std::sync::{Arc, Mutex};

    #[test]
    fn given_input_stream_when_fan_out_graph_runs_with_aeron_rs_then_publishes_sum_and_count() {
        let _driver = MediaDriverGuard::start()
            .expect("Failed to start media driver - see error message for installation instructions");

        let context = Context::new();
        let mut aeron = Aeron::new(context).expect("Failed to connect to Aeron");

        let channel = CString::new("aeron:ipc").unwrap();
        let input_stream = 2001;
        let sum_stream = 2002;
        let count_stream = 2003;

        // Helper to add publication and wait
        fn add_pub(aeron: &mut Aeron, channel: CString, stream_id: i32) -> Arc<Mutex<aeron_rs::publication::Publication>> {
            let id = aeron.add_publication(channel, stream_id).expect("Failed to add pub");
            loop {
                if let Ok(pub_arc) = aeron.find_publication(id) {
                    return pub_arc;
                }
                thread::sleep(Duration::from_millis(10));
            }
        }

        // Helper to add subscription and wait
        fn add_sub(aeron: &mut Aeron, channel: CString, stream_id: i32) -> Arc<Mutex<aeron_rs::subscription::Subscription>> {
            let id = aeron.add_subscription(channel, stream_id).expect("Failed to add sub");
            loop {
                if let Ok(sub_arc) = aeron.find_subscription(id) {
                    return sub_arc;
                }
                thread::sleep(Duration::from_millis(10));
            }
        }

        let input_publisher = AeronRsPublisher::new(add_pub(&mut aeron, channel.clone(), input_stream));
        let input_subscriber = AeronRsSubscriber::new(add_sub(&mut aeron, channel.clone(), input_stream));
        let sum_publisher = AeronRsPublisher::new(add_pub(&mut aeron, channel.clone(), sum_stream));
        let sum_subscriber = AeronRsSubscriber::new(add_sub(&mut aeron, channel.clone(), sum_stream));
        let count_publisher = AeronRsPublisher::new(add_pub(&mut aeron, channel.clone(), count_stream));
        let count_subscriber = AeronRsSubscriber::new(add_sub(&mut aeron, channel.clone(), count_stream));

        run_fan_out_test(
            input_publisher,
            input_subscriber,
            sum_publisher,
            sum_subscriber,
            count_publisher,
            count_subscriber,
        );
    }
}
