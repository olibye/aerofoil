//! Combined pub/sub (transceiver) benchmarks for Aeron transport.
//!
//! Measures performance when publishing and subscribing concurrently,
//! including request/response roundtrip latency and bidirectional exchange.

mod common;

use common::MessageSize;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::thread;
use std::time::Duration;

#[cfg(feature = "rusteron")]
mod rusteron_bench {
    use super::*;
    use aerofoil::transport::rusteron::{RusteronPublisher, RusteronSubscriber};
    use aerofoil::transport::{AeronPublisher, AeronSubscriber};
    use common::rusteron_support::BenchContext;

    /// Benchmark simultaneous publish/subscribe on different streams.
    pub fn bench_simultaneous_pub_sub(c: &mut Criterion) {
        let ctx = match BenchContext::new() {
            Some(c) => c,
            None => return,
        };

        let mut group = c.benchmark_group("rusteron/transceiver/simultaneous");

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            let publication = ctx.add_publication(7001);
            let subscription = ctx.add_subscription(7002);

            let mut publisher = RusteronPublisher::new(publication);
            let mut subscriber = RusteronSubscriber::new(subscription);

            group.throughput(Throughput::Bytes(size.bytes() as u64 * 2)); // pub + sub

            group.bench_function(BenchmarkId::from_parameter(size.name()), |b| {
                let buffer = size.create_buffer();

                b.iter(|| {
                    // Publish
                    let pub_result = publisher.offer(&buffer);

                    // Poll (may or may not have messages)
                    let poll_result = subscriber.poll(|_fragment| Ok(()));

                    black_box((pub_result, poll_result))
                });
            });
        }

        group.finish();
    }

    /// Benchmark request/response roundtrip latency.
    pub fn bench_request_response(c: &mut Criterion) {
        let ctx = match BenchContext::new() {
            Some(c) => c,
            None => return,
        };

        let mut group = c.benchmark_group("rusteron/transceiver/roundtrip");

        for size in [MessageSize::Small, MessageSize::Medium] {
            // Client: publishes requests, subscribes to responses
            let client_pub = ctx.add_publication(8001);
            let client_sub = ctx.add_subscription(8002);
            let mut client_publisher = RusteronPublisher::new(client_pub);
            let mut client_subscriber = RusteronSubscriber::new(client_sub);

            // Server: subscribes to requests, publishes responses
            let server_sub = ctx.add_subscription(8001);
            let server_pub = ctx.add_publication(8002);
            let mut server_subscriber = RusteronSubscriber::new(server_sub);
            let mut server_publisher = RusteronPublisher::new(server_pub);

            thread::sleep(Duration::from_millis(100));

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
        }

        group.finish();
    }

    /// Benchmark bidirectional symmetric exchange.
    pub fn bench_bidirectional(c: &mut Criterion) {
        let ctx = match BenchContext::new() {
            Some(c) => c,
            None => return,
        };

        let mut group = c.benchmark_group("rusteron/transceiver/bidirectional");

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            // Side A: pub to B, sub from B
            let pub_a = ctx.add_publication(9001);
            let sub_a = ctx.add_subscription(9002);
            let mut publisher_a = RusteronPublisher::new(pub_a);
            let mut subscriber_a = RusteronSubscriber::new(sub_a);

            // Side B: pub to A, sub from A
            let pub_b = ctx.add_publication(9002);
            let sub_b = ctx.add_subscription(9001);
            let mut publisher_b = RusteronPublisher::new(pub_b);
            let mut subscriber_b = RusteronSubscriber::new(sub_b);

            thread::sleep(Duration::from_millis(100));

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
        }

        group.finish();
    }
}

#[cfg(feature = "aeron-rs")]
mod aeron_rs_bench {
    use super::*;
    use aeron_rs::concurrent::atomic_buffer::AtomicBuffer;
    use common::aeron_rs_support::BenchContext;

    /// Benchmark simultaneous publish/subscribe on different streams.
    pub fn bench_simultaneous_pub_sub(c: &mut Criterion) {
        let mut ctx = match BenchContext::new() {
            Some(c) => c,
            None => return,
        };

        let mut group = c.benchmark_group("aeron-rs/transceiver/simultaneous");

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            // Use stream IDs in 17000+ range to avoid conflicts with rusteron (7000-7999)
            let publication = ctx.add_publication(17001);
            let subscription = ctx.add_subscription(17002);

            group.throughput(Throughput::Bytes(size.bytes() as u64 * 2));

            group.bench_function(BenchmarkId::from_parameter(size.name()), |b| {
                let mut buffer = size.create_buffer();

                b.iter(|| {
                    // Publish
                    let pub_result = {
                        let atomic_buffer = AtomicBuffer::wrap_slice(&mut buffer);
                        let pub_guard = publication.lock().expect("Mutex poisoned");
                        pub_guard.offer(atomic_buffer)
                    };

                    // Poll
                    let poll_result = {
                        let mut sub_guard = subscription.lock().expect("Mutex poisoned");
                        sub_guard.poll(&mut |_, _, _, _| {}, 1)
                    };

                    black_box((pub_result, poll_result))
                });
            });
        }

        group.finish();
    }

    /// Benchmark request/response roundtrip latency.
    pub fn bench_request_response(c: &mut Criterion) {
        let mut ctx = match BenchContext::new() {
            Some(c) => c,
            None => return,
        };

        let mut group = c.benchmark_group("aeron-rs/transceiver/roundtrip");

        for size in [MessageSize::Small, MessageSize::Medium] {
            // Client: publishes requests, subscribes to responses
            // Use stream IDs in 18000+ range to avoid conflicts with rusteron (8000-8999)
            let client_pub = ctx.add_publication(18001);
            let client_sub = ctx.add_subscription(18002);

            // Server: subscribes to requests, publishes responses
            let server_sub = ctx.add_subscription(18001);
            let server_pub = ctx.add_publication(18002);

            thread::sleep(Duration::from_millis(100));

            group.throughput(Throughput::Elements(1));

            group.bench_function(BenchmarkId::from_parameter(size.name()), |b| {
                let mut request = size.create_buffer();
                let mut response = size.create_buffer();

                b.iter(|| {
                    // Client sends request
                    loop {
                        let atomic_buffer = AtomicBuffer::wrap_slice(&mut request);
                        let pub_guard = client_pub.lock().expect("Mutex poisoned");
                        if pub_guard.offer(atomic_buffer).is_ok() {
                            break;
                        }
                        thread::yield_now();
                    }

                    // Server polls for request
                    let mut received_request = false;
                    for _ in 0..1000 {
                        let mut sub_guard = server_sub.lock().expect("Mutex poisoned");
                        let count = sub_guard.poll(&mut |_, _, _, _| {}, 1);
                        if count > 0 {
                            received_request = true;
                            break;
                        }
                        thread::yield_now();
                    }

                    if received_request {
                        // Server sends response
                        loop {
                            let atomic_buffer = AtomicBuffer::wrap_slice(&mut response);
                            let pub_guard = server_pub.lock().expect("Mutex poisoned");
                            if pub_guard.offer(atomic_buffer).is_ok() {
                                break;
                            }
                            thread::yield_now();
                        }

                        // Client polls for response
                        for _ in 0..1000 {
                            let mut sub_guard = client_sub.lock().expect("Mutex poisoned");
                            let count = sub_guard.poll(&mut |_, _, _, _| {}, 1);
                            if count > 0 {
                                break;
                            }
                            thread::yield_now();
                        }
                    }
                });
            });
        }

        group.finish();
    }

    /// Benchmark bidirectional symmetric exchange.
    pub fn bench_bidirectional(c: &mut Criterion) {
        let mut ctx = match BenchContext::new() {
            Some(c) => c,
            None => return,
        };

        let mut group = c.benchmark_group("aeron-rs/transceiver/bidirectional");

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            // Side A: pub to B, sub from B
            // Use stream IDs in 19000+ range to avoid conflicts with rusteron (9000-9999)
            let pub_a = ctx.add_publication(19001);
            let sub_a = ctx.add_subscription(19002);

            // Side B: pub to A, sub from A
            let pub_b = ctx.add_publication(19002);
            let sub_b = ctx.add_subscription(19001);

            thread::sleep(Duration::from_millis(100));

            group.throughput(Throughput::Bytes(size.bytes() as u64 * 2));

            group.bench_function(BenchmarkId::from_parameter(size.name()), |b| {
                let mut buffer_a = size.create_buffer();
                let mut buffer_b = size.create_buffer();

                b.iter(|| {
                    // Both sides publish
                    {
                        let atomic_buffer = AtomicBuffer::wrap_slice(&mut buffer_a);
                        let pub_guard = pub_a.lock().expect("Mutex poisoned");
                        let _ = pub_guard.offer(atomic_buffer);
                    }
                    {
                        let atomic_buffer = AtomicBuffer::wrap_slice(&mut buffer_b);
                        let pub_guard = pub_b.lock().expect("Mutex poisoned");
                        let _ = pub_guard.offer(atomic_buffer);
                    }

                    // Both sides poll
                    {
                        let mut sub_guard = sub_a.lock().expect("Mutex poisoned");
                        let _ = sub_guard.poll(&mut |_, _, _, _| {}, 1);
                    }
                    {
                        let mut sub_guard = sub_b.lock().expect("Mutex poisoned");
                        let _ = sub_guard.poll(&mut |_, _, _, _| {}, 1);
                    }
                });
            });
        }

        group.finish();
    }
}

// When both features enabled, run both benchmark suites
#[cfg(all(feature = "rusteron", feature = "aeron-rs"))]
criterion_group!(
    benches,
    rusteron_bench::bench_simultaneous_pub_sub,
    rusteron_bench::bench_request_response,
    rusteron_bench::bench_bidirectional,
    aeron_rs_bench::bench_simultaneous_pub_sub,
    aeron_rs_bench::bench_request_response,
    aeron_rs_bench::bench_bidirectional
);

// When only rusteron enabled
#[cfg(all(feature = "rusteron", not(feature = "aeron-rs")))]
criterion_group!(
    benches,
    rusteron_bench::bench_simultaneous_pub_sub,
    rusteron_bench::bench_request_response,
    rusteron_bench::bench_bidirectional
);

// When only aeron-rs enabled
#[cfg(all(feature = "aeron-rs", not(feature = "rusteron")))]
criterion_group!(
    benches,
    aeron_rs_bench::bench_simultaneous_pub_sub,
    aeron_rs_bench::bench_request_response,
    aeron_rs_bench::bench_bidirectional
);

#[cfg(not(any(feature = "rusteron", feature = "aeron-rs")))]
fn no_backend(_c: &mut Criterion) {
    eprintln!("Benchmarks require 'rusteron' or 'aeron-rs' feature.");
}

#[cfg(not(any(feature = "rusteron", feature = "aeron-rs")))]
criterion_group!(benches, no_backend);

criterion_main!(benches);
