//! Combined pub/sub (transceiver) benchmarks for Aeron transport.
//!
//! Measures performance when publishing and subscribing concurrently,
//! including request/response roundtrip latency and bidirectional exchange.

mod common;

use aerofoil::transport::{AeronPublisher, AeronSubscriber};
use common::{MessageSize, CHANNEL};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::thread;
use std::time::Duration;

#[cfg(feature = "rusteron")]
mod rusteron_bench {
    use super::*;
    use aerofoil::transport::rusteron::{RusteronPublisher, RusteronSubscriber};
    use common::MediaDriverGuard;
    use rusteron_client::IntoCString;

    struct TransceiverSetup {
        _driver: MediaDriverGuard,
        publisher: RusteronPublisher,
        subscriber: RusteronSubscriber,
    }

    fn setup_transceiver(pub_stream: i32, sub_stream: i32) -> Option<TransceiverSetup> {
        let driver = match MediaDriverGuard::start() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Skipping benchmark: {}", e);
                return None;
            }
        };

        let context = rusteron_client::AeronContext::new().expect("Failed to create context");
        let aeron = rusteron_client::Aeron::new(&context).expect("Failed to create Aeron");
        aeron.start().expect("Failed to start Aeron");

        // Create publisher
        let async_pub = aeron
            .async_add_publication(&CHANNEL.into_c_string(), pub_stream)
            .expect("Failed to start publication");

        let publication = async_pub
            .poll_blocking(Duration::from_secs(5))
            .expect("Failed to complete publication");

        let publisher = RusteronPublisher::new(publication);

        // Create subscriber on different stream
        let async_sub = aeron
            .async_add_subscription(
                &CHANNEL.into_c_string(),
                sub_stream,
                rusteron_client::Handlers::no_available_image_handler(),
                rusteron_client::Handlers::no_unavailable_image_handler(),
            )
            .expect("Failed to start subscription");

        let subscription = async_sub
            .poll_blocking(Duration::from_secs(5))
            .expect("Failed to complete subscription");

        let subscriber = RusteronSubscriber::new(subscription);

        thread::sleep(Duration::from_millis(100));

        Some(TransceiverSetup {
            _driver: driver,
            publisher,
            subscriber,
        })
    }

    /// Benchmark simultaneous publish/subscribe on different streams.
    pub fn bench_simultaneous_pub_sub(c: &mut Criterion) {
        let mut group = c.benchmark_group("rusteron/transceiver/simultaneous");

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            let pub_stream = 7001;
            let sub_stream = 7002;

            let mut setup = match setup_transceiver(pub_stream, sub_stream) {
                Some(s) => s,
                None => return,
            };

            group.throughput(Throughput::Bytes(size.bytes() as u64 * 2)); // pub + sub

            group.bench_function(BenchmarkId::from_parameter(size.name()), |b| {
                let buffer = size.create_buffer();

                b.iter(|| {
                    // Publish
                    let pub_result = setup.publisher.offer(&buffer);

                    // Poll (may or may not have messages)
                    let poll_result = setup.subscriber.poll(|_fragment| Ok(()));

                    black_box((pub_result, poll_result))
                });
            });
        }

        group.finish();
    }

    /// Benchmark request/response roundtrip latency.
    ///
    /// Simulates a request-response pattern where:
    /// 1. Client sends request on stream A
    /// 2. Server receives on stream A, sends response on stream B
    /// 3. Client receives response on stream B
    pub fn bench_request_response(c: &mut Criterion) {
        let mut group = c.benchmark_group("rusteron/transceiver/roundtrip");

        for size in [MessageSize::Small, MessageSize::Medium] {
            // Skip large for latency test
            // Request stream and response stream
            let request_stream = 8001;
            let response_stream = 8002;

            let driver = match MediaDriverGuard::start() {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Skipping benchmark: {}", e);
                    return;
                }
            };

            let context = rusteron_client::AeronContext::new().expect("Failed to create context");
            let aeron = rusteron_client::Aeron::new(&context).expect("Failed to create Aeron");
            aeron.start().expect("Failed to start Aeron");

            // Client: publishes requests, subscribes to responses
            let async_pub = aeron
                .async_add_publication(&CHANNEL.into_c_string(), request_stream)
                .expect("Failed to start publication");
            let client_pub = async_pub
                .poll_blocking(Duration::from_secs(5))
                .expect("Failed to complete publication");
            let mut client_publisher = RusteronPublisher::new(client_pub);

            let async_sub = aeron
                .async_add_subscription(
                    &CHANNEL.into_c_string(),
                    response_stream,
                    rusteron_client::Handlers::no_available_image_handler(),
                    rusteron_client::Handlers::no_unavailable_image_handler(),
                )
                .expect("Failed to start subscription");
            let client_sub = async_sub
                .poll_blocking(Duration::from_secs(5))
                .expect("Failed to complete subscription");
            let mut client_subscriber = RusteronSubscriber::new(client_sub);

            // Server: subscribes to requests, publishes responses
            let async_sub = aeron
                .async_add_subscription(
                    &CHANNEL.into_c_string(),
                    request_stream,
                    rusteron_client::Handlers::no_available_image_handler(),
                    rusteron_client::Handlers::no_unavailable_image_handler(),
                )
                .expect("Failed to start subscription");
            let server_sub = async_sub
                .poll_blocking(Duration::from_secs(5))
                .expect("Failed to complete subscription");
            let mut server_subscriber = RusteronSubscriber::new(server_sub);

            let async_pub = aeron
                .async_add_publication(&CHANNEL.into_c_string(), response_stream)
                .expect("Failed to start publication");
            let server_pub = async_pub
                .poll_blocking(Duration::from_secs(5))
                .expect("Failed to complete publication");
            let mut server_publisher = RusteronPublisher::new(server_pub);

            thread::sleep(Duration::from_millis(200));

            group.throughput(Throughput::Elements(1));

            group.bench_function(BenchmarkId::from_parameter(size.name()), |b| {
                let request = size.create_buffer();
                let response = size.create_buffer();

                b.iter(|| {
                    // Client sends request
                    while client_publisher.offer(&request).is_err() {
                        thread::yield_now();
                    }

                    // Server polls for request
                    let mut received_request = false;
                    for _ in 0..1000 {
                        let count = server_subscriber.poll(|_| Ok(())).unwrap_or(0);
                        if count > 0 {
                            received_request = true;
                            break;
                        }
                        thread::yield_now();
                    }

                    if received_request {
                        // Server sends response
                        while server_publisher.offer(&response).is_err() {
                            thread::yield_now();
                        }

                        // Client polls for response
                        for _ in 0..1000 {
                            let count = client_subscriber.poll(|_| Ok(())).unwrap_or(0);
                            if count > 0 {
                                break;
                            }
                            thread::yield_now();
                        }
                    }
                });
            });

            drop(driver);
        }

        group.finish();
    }

    /// Benchmark bidirectional symmetric exchange.
    ///
    /// Both sides simultaneously publish and subscribe, measuring
    /// throughput of bidirectional communication.
    pub fn bench_bidirectional(c: &mut Criterion) {
        let mut group = c.benchmark_group("rusteron/transceiver/bidirectional");

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            let stream_a_to_b = 9001;
            let stream_b_to_a = 9002;

            let driver = match MediaDriverGuard::start() {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Skipping benchmark: {}", e);
                    return;
                }
            };

            let context = rusteron_client::AeronContext::new().expect("Failed to create context");
            let aeron = rusteron_client::Aeron::new(&context).expect("Failed to create Aeron");
            aeron.start().expect("Failed to start Aeron");

            // Side A: pub to B, sub from B
            let async_pub = aeron
                .async_add_publication(&CHANNEL.into_c_string(), stream_a_to_b)
                .expect("Failed");
            let pub_a = async_pub
                .poll_blocking(Duration::from_secs(5))
                .expect("Failed");
            let mut publisher_a = RusteronPublisher::new(pub_a);

            let async_sub = aeron
                .async_add_subscription(
                    &CHANNEL.into_c_string(),
                    stream_b_to_a,
                    rusteron_client::Handlers::no_available_image_handler(),
                    rusteron_client::Handlers::no_unavailable_image_handler(),
                )
                .expect("Failed");
            let sub_a = async_sub
                .poll_blocking(Duration::from_secs(5))
                .expect("Failed");
            let mut subscriber_a = RusteronSubscriber::new(sub_a);

            // Side B: pub to A, sub from A
            let async_pub = aeron
                .async_add_publication(&CHANNEL.into_c_string(), stream_b_to_a)
                .expect("Failed");
            let pub_b = async_pub
                .poll_blocking(Duration::from_secs(5))
                .expect("Failed");
            let mut publisher_b = RusteronPublisher::new(pub_b);

            let async_sub = aeron
                .async_add_subscription(
                    &CHANNEL.into_c_string(),
                    stream_a_to_b,
                    rusteron_client::Handlers::no_available_image_handler(),
                    rusteron_client::Handlers::no_unavailable_image_handler(),
                )
                .expect("Failed");
            let sub_b = async_sub
                .poll_blocking(Duration::from_secs(5))
                .expect("Failed");
            let mut subscriber_b = RusteronSubscriber::new(sub_b);

            thread::sleep(Duration::from_millis(200));

            group.throughput(Throughput::Bytes(size.bytes() as u64 * 2));

            group.bench_function(BenchmarkId::from_parameter(size.name()), |b| {
                let buffer_a = size.create_buffer();
                let buffer_b = size.create_buffer();

                b.iter(|| {
                    // Both sides publish
                    let _ = publisher_a.offer(&buffer_a);
                    let _ = publisher_b.offer(&buffer_b);

                    // Both sides poll
                    let _ = subscriber_a.poll(|_| Ok(()));
                    let _ = subscriber_b.poll(|_| Ok(()));
                });
            });

            drop(driver);
        }

        group.finish();
    }
}

#[cfg(feature = "aeron-rs")]
mod aeron_rs_bench {
    use super::*;
    use aerofoil::transport::aeron_rs::{AeronRsPublisher, AeronRsSubscriber};
    use aeron_rs::client::Client;
    use aeron_rs::context::Context;

    pub fn bench_simultaneous_pub_sub(c: &mut Criterion) {
        let context = Context::new();
        let mut client = Client::connect(context).expect("Failed to connect to media driver");

        let mut group = c.benchmark_group("aeron-rs/transceiver/simultaneous");

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            let pub_stream = 10001;
            let sub_stream = 10002;

            let publication = client
                .add_publication(CHANNEL, pub_stream)
                .expect("Failed to add publication");
            let subscription = client
                .add_subscription(CHANNEL, sub_stream)
                .expect("Failed to add subscription");

            let mut publisher = AeronRsPublisher::new(publication);
            let mut subscriber = AeronRsSubscriber::new(subscription);

            thread::sleep(Duration::from_millis(100));

            group.throughput(Throughput::Bytes(size.bytes() as u64 * 2));

            group.bench_function(BenchmarkId::from_parameter(size.name()), |b| {
                let buffer = size.create_buffer();

                b.iter(|| {
                    let pub_result = publisher.offer(&buffer);
                    let poll_result = subscriber.poll(|_fragment| Ok(()));
                    black_box((pub_result, poll_result))
                });
            });
        }

        group.finish();
    }
}

#[cfg(feature = "rusteron")]
criterion_group!(
    benches,
    rusteron_bench::bench_simultaneous_pub_sub,
    rusteron_bench::bench_request_response,
    rusteron_bench::bench_bidirectional
);

#[cfg(all(feature = "aeron-rs", not(feature = "rusteron")))]
criterion_group!(benches, aeron_rs_bench::bench_simultaneous_pub_sub);

#[cfg(not(any(feature = "rusteron", feature = "aeron-rs")))]
fn no_backend(_c: &mut Criterion) {
    eprintln!("No Aeron backend enabled. Enable 'rusteron' or 'aeron-rs' feature.");
}

#[cfg(not(any(feature = "rusteron", feature = "aeron-rs")))]
criterion_group!(benches, no_backend);

criterion_main!(benches);
