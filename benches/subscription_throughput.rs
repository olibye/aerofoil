//! Subscription throughput benchmarks for Aeron transport.
//!
//! Measures the throughput of receiving messages using AeronSubscriberNode
//! with Wingfoil graph execution. Benchmarks use the same patterns as the
//! integration tests (summing_node_test.rs).

mod common;

use aerofoil::transport::AeronPublisher;
use common::{MessageSize, CHANNEL};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::thread;
use std::time::Duration;

#[cfg(feature = "rusteron")]
mod rusteron_bench {
    use super::*;
    use aerofoil::transport::rusteron::{RusteronPublisher, RusteronSubscriber};
    use aerofoil::transport::AeronSubscriber;
    use common::MediaDriverGuard;
    use rusteron_client::IntoCString;

    fn setup_pub_sub(
        aeron: &rusteron_client::Aeron,
        stream_id: i32,
    ) -> (RusteronPublisher, RusteronSubscriber) {
        let async_pub = aeron
            .async_add_publication(&CHANNEL.into_c_string(), stream_id)
            .expect("Failed to start publication");

        let publication = async_pub
            .poll_blocking(Duration::from_secs(5))
            .expect("Failed to complete publication");

        let publisher = RusteronPublisher::new(publication);

        let async_sub = aeron
            .async_add_subscription(
                &CHANNEL.into_c_string(),
                stream_id,
                rusteron_client::Handlers::no_available_image_handler(),
                rusteron_client::Handlers::no_unavailable_image_handler(),
            )
            .expect("Failed to start subscription");

        let subscription = async_sub
            .poll_blocking(Duration::from_secs(5))
            .expect("Failed to complete subscription");

        let subscriber = RusteronSubscriber::new(subscription);

        thread::sleep(Duration::from_millis(100));

        (publisher, subscriber)
    }

    /// Run all rusteron subscription benchmarks with a single shared media driver.
    pub fn bench_all(c: &mut Criterion) {
        let _driver = match MediaDriverGuard::start() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Skipping benchmark: {}", e);
                return;
            }
        };

        let context = rusteron_client::AeronContext::new().expect("Failed to create context");
        let aeron = rusteron_client::Aeron::new(&context).expect("Failed to create Aeron");
        aeron.start().expect("Failed to start Aeron");

        bench_poll_throughput(c, &aeron);
        bench_poll_with_parsing(c, &aeron);
        bench_burst_throughput(c, &aeron);
    }

    /// Benchmark raw poll performance.
    ///
    /// This measures the throughput of the poll operation itself,
    /// which is the core subscription hot path.
    fn bench_poll_throughput(c: &mut Criterion, aeron: &rusteron_client::Aeron) {
        let mut group = c.benchmark_group("rusteron/poll");
        // Limit times to avoid media driver service interval timeout
        group.warm_up_time(Duration::from_millis(500));
        group.measurement_time(Duration::from_secs(1));
        group.sample_size(10);

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            let stream_id = 5001 + size.bytes() as i32;

            let (mut publisher, mut subscriber) = setup_pub_sub(aeron, stream_id);

            group.throughput(Throughput::Bytes(size.bytes() as u64));

            group.bench_function(BenchmarkId::from_parameter(size.name()), |b| {
                let buffer = size.create_buffer();

                b.iter(|| {
                    // Publish a message
                    let _ = publisher.offer(&buffer);

                    // Poll for it
                    let count = subscriber.poll(|_fragment| Ok(())).unwrap_or(0);

                    black_box(count)
                });
            });
        }

        group.finish();
    }

    /// Benchmark subscription with message parsing.
    ///
    /// This measures the throughput when including message parsing overhead,
    /// simulating real-world usage where messages need to be deserialized.
    fn bench_poll_with_parsing(c: &mut Criterion, aeron: &rusteron_client::Aeron) {
        let mut group = c.benchmark_group("rusteron/poll_parse");
        // Limit times to avoid media driver service interval timeout
        group.warm_up_time(Duration::from_millis(500));
        group.measurement_time(Duration::from_secs(2));
        group.sample_size(20);

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            let stream_id = 5101 + size.bytes() as i32;

            let (mut publisher, mut subscriber) = setup_pub_sub(aeron, stream_id);

            group.throughput(Throughput::Bytes(size.bytes() as u64));

            group.bench_function(BenchmarkId::from_parameter(size.name()), |b| {
                let mut buffer = size.create_buffer();
                // Write a value at the start of the buffer
                buffer[0..8].copy_from_slice(&42i64.to_le_bytes());

                b.iter(|| {
                    // Publish a message
                    let _ = publisher.offer(&buffer);

                    // Poll and parse
                    let mut value: i64 = 0;
                    let count = subscriber
                        .poll(|fragment| {
                            if fragment.len() >= 8 {
                                let bytes: [u8; 8] = fragment[0..8].try_into().unwrap();
                                value = i64::from_le_bytes(bytes);
                            }
                            Ok(())
                        })
                        .unwrap_or(0);

                    black_box((count, value))
                });
            });
        }

        group.finish();
    }

    /// Benchmark burst subscription throughput.
    ///
    /// Measures throughput when processing multiple messages per poll,
    /// which is more representative of high-frequency trading scenarios.
    fn bench_burst_throughput(c: &mut Criterion, aeron: &rusteron_client::Aeron) {
        let mut group = c.benchmark_group("rusteron/poll_burst");
        // Limit times to avoid media driver service interval timeout
        group.warm_up_time(Duration::from_millis(500));
        group.measurement_time(Duration::from_secs(2));
        group.sample_size(20);
        let burst_size = 100;

        for size in [MessageSize::Small, MessageSize::Medium] {
            let stream_id = 5201 + size.bytes() as i32;

            let (mut publisher, mut subscriber) = setup_pub_sub(aeron, stream_id);

            group.throughput(Throughput::Elements(burst_size as u64));

            group.bench_function(BenchmarkId::from_parameter(size.name()), |b| {
                let buffer = size.create_buffer();

                b.iter(|| {
                    // Publish burst of messages
                    for _ in 0..burst_size {
                        while publisher.offer(&buffer).is_err() {
                            thread::yield_now();
                        }
                    }

                    // Poll until all received
                    let mut total = 0;
                    while total < burst_size {
                        let count = subscriber.poll(|_| Ok(())).unwrap_or(0);
                        total += count;
                        if count == 0 {
                            thread::yield_now();
                        }
                    }

                    black_box(total)
                });
            });
        }

        group.finish();
    }
}

#[cfg(feature = "aeron-rs")]
mod aeron_rs_bench {
    use super::*;
    use aerofoil::transport::aeron_rs::{AeronRsPublisher, AeronRsSubscriber};
    use aerofoil::transport::AeronSubscriber;
    use aeron_rs::client::Client;
    use aeron_rs::context::Context;

    pub fn bench_poll_throughput(c: &mut Criterion) {
        let context = Context::new();
        let mut client = Client::connect(context).expect("Failed to connect to media driver");

        let mut group = c.benchmark_group("aeron-rs/poll");

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            let stream_id = 6001 + size.bytes() as i32;

            let publication = client
                .add_publication(CHANNEL, stream_id)
                .expect("Failed to add publication");

            let subscription = client
                .add_subscription(CHANNEL, stream_id)
                .expect("Failed to add subscription");

            let mut publisher = AeronRsPublisher::new(publication);
            let mut subscriber = AeronRsSubscriber::new(subscription);

            thread::sleep(Duration::from_millis(100));

            group.throughput(Throughput::Bytes(size.bytes() as u64));

            group.bench_function(BenchmarkId::from_parameter(size.name()), |b| {
                let buffer = size.create_buffer();

                b.iter(|| {
                    // Publish a message
                    let _ = publisher.offer(&buffer);

                    // Poll for it
                    let count = subscriber.poll(|_fragment| Ok(())).unwrap_or(0);

                    black_box(count)
                });
            });
        }

        group.finish();
    }

    pub fn bench_poll_with_parsing(c: &mut Criterion) {
        let context = Context::new();
        let mut client = Client::connect(context).expect("Failed to connect to media driver");

        let mut group = c.benchmark_group("aeron-rs/poll_parse");

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            let stream_id = 6101 + size.bytes() as i32;

            let publication = client
                .add_publication(CHANNEL, stream_id)
                .expect("Failed to add publication");

            let subscription = client
                .add_subscription(CHANNEL, stream_id)
                .expect("Failed to add subscription");

            let mut publisher = AeronRsPublisher::new(publication);
            let mut subscriber = AeronRsSubscriber::new(subscription);

            thread::sleep(Duration::from_millis(100));

            group.throughput(Throughput::Bytes(size.bytes() as u64));

            group.bench_function(BenchmarkId::from_parameter(size.name()), |b| {
                let mut buffer = size.create_buffer();
                buffer[0..8].copy_from_slice(&42i64.to_le_bytes());

                b.iter(|| {
                    let _ = publisher.offer(&buffer);

                    let mut value: i64 = 0;
                    let count = subscriber
                        .poll(|fragment| {
                            if fragment.len() >= 8 {
                                let bytes: [u8; 8] = fragment[0..8].try_into().unwrap();
                                value = i64::from_le_bytes(bytes);
                            }
                            Ok(())
                        })
                        .unwrap_or(0);

                    black_box((count, value))
                });
            });
        }

        group.finish();
    }
}

#[cfg(feature = "rusteron")]
criterion_group!(benches, rusteron_bench::bench_all);

#[cfg(all(feature = "aeron-rs", not(feature = "rusteron")))]
criterion_group!(
    benches,
    aeron_rs_bench::bench_poll_throughput,
    aeron_rs_bench::bench_poll_with_parsing
);

#[cfg(not(any(feature = "rusteron", feature = "aeron-rs")))]
fn no_backend(_c: &mut Criterion) {
    eprintln!("No Aeron backend enabled. Enable 'rusteron' or 'aeron-rs' feature.");
}

#[cfg(not(any(feature = "rusteron", feature = "aeron-rs")))]
criterion_group!(benches, no_backend);

criterion_main!(benches);
