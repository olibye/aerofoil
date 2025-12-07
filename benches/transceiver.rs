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
    use common::rusteron_support::BenchContext;
    use common::generic;

    /// Benchmark simultaneous publish/subscribe on different streams.
    pub fn bench_simultaneous_pub_sub(c: &mut Criterion) {
        let ctx = match BenchContext::new() {
            Some(c) => c,
            None => return,
        };

        let publication = ctx.add_publication(7001);
        let subscription = ctx.add_subscription(7002);
        let mut publisher = RusteronPublisher::new(publication);
        let mut subscriber = RusteronSubscriber::new(subscription);

        generic::bench_simultaneous_pub_sub(c, &mut publisher, &mut subscriber, "rusteron");
    }

    /// Benchmark request/response roundtrip latency.
    pub fn bench_request_response(c: &mut Criterion) {
        let ctx = match BenchContext::new() {
            Some(c) => c,
            None => return,
        };

        let client_pub = ctx.add_publication(8001);
        let client_sub = ctx.add_subscription(8002);
        let mut client_publisher = RusteronPublisher::new(client_pub);
        let mut client_subscriber = RusteronSubscriber::new(client_sub);

        let server_sub = ctx.add_subscription(8001);
        let server_pub = ctx.add_publication(8002);
        let mut server_subscriber = RusteronSubscriber::new(server_sub);
        let mut server_publisher = RusteronPublisher::new(server_pub);

        generic::bench_request_response(
            c,
            &mut client_publisher,
            &mut client_subscriber,
            &mut server_publisher,
            &mut server_subscriber,
            "rusteron",
        );
    }

    /// Benchmark bidirectional symmetric exchange.
    pub fn bench_bidirectional(c: &mut Criterion) {
        let ctx = match BenchContext::new() {
            Some(c) => c,
            None => return,
        };

        let pub_a = ctx.add_publication(9001);
        let sub_a = ctx.add_subscription(9002);
        let mut publisher_a = RusteronPublisher::new(pub_a);
        let mut subscriber_a = RusteronSubscriber::new(sub_a);

        let pub_b = ctx.add_publication(9002);
        let sub_b = ctx.add_subscription(9001);
        let mut publisher_b = RusteronPublisher::new(pub_b);
        let mut subscriber_b = RusteronSubscriber::new(sub_b);

        generic::bench_bidirectional(
            c,
            &mut publisher_a,
            &mut subscriber_a,
            &mut publisher_b,
            &mut subscriber_b,
            "rusteron",
        );
    }
}

#[cfg(feature = "aeron-rs")]
mod aeron_rs_bench {
    use super::*;
    use aerofoil::transport::aeron_rs::{AeronRsPublisher, AeronRsSubscriber};
    use aeron_rs::concurrent::atomic_buffer::AtomicBuffer;
    use common::aeron_rs_support::BenchContext;
    use common::generic;

    /// Benchmark simultaneous publish/subscribe on different streams.
    pub fn bench_simultaneous_pub_sub(c: &mut Criterion) {
        let mut ctx = match BenchContext::new() {
            Some(c) => c,
            None => return,
        };

        // Bare
        bench_simultaneous_pub_sub_bare(c, &mut ctx);

        // Aerofoil
        bench_simultaneous_pub_sub_aerofoil(c, &mut ctx);
    }

    fn bench_simultaneous_pub_sub_aerofoil(c: &mut Criterion, ctx: &mut BenchContext) {
        let publication = ctx.add_publication(7001);
        let subscription = ctx.add_subscription(7002);
        let mut publisher = AeronRsPublisher::new(publication);
        let mut subscriber = AeronRsSubscriber::new(subscription);

        generic::bench_simultaneous_pub_sub(c, &mut publisher, &mut subscriber, "aeron-rs");
    }

    fn bench_simultaneous_pub_sub_bare(c: &mut Criterion, ctx: &mut BenchContext) {
        let mut group = c.benchmark_group("aeron-rs/transceiver/simultaneous/bare");

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            let publication = ctx.add_publication(7001);
            let subscription = ctx.add_subscription(7002);

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

    pub fn bench_request_response(c: &mut Criterion) {
        let mut ctx = match BenchContext::new() {
            Some(c) => c,
            None => return,
        };

        // Bare
        bench_request_response_bare(c, &mut ctx);

        // Aerofoil
        bench_request_response_aerofoil(c, &mut ctx);
    }

    fn bench_request_response_aerofoil(c: &mut Criterion, ctx: &mut BenchContext) {
        let client_pub = ctx.add_publication(8001);
        let client_sub = ctx.add_subscription(8002);
        let mut client_publisher = AeronRsPublisher::new(client_pub);
        let mut client_subscriber = AeronRsSubscriber::new(client_sub);

        let server_sub = ctx.add_subscription(8001);
        let server_pub = ctx.add_publication(8002);
        let mut server_subscriber = AeronRsSubscriber::new(server_sub);
        let mut server_publisher = AeronRsPublisher::new(server_pub);

        generic::bench_request_response(
            c,
            &mut client_publisher,
            &mut client_subscriber,
            &mut server_publisher,
            &mut server_subscriber,
            "aeron-rs",
        );
    }

    fn bench_request_response_bare(c: &mut Criterion, ctx: &mut BenchContext) {
        let mut group = c.benchmark_group("aeron-rs/transceiver/roundtrip/bare");

        for size in [MessageSize::Small, MessageSize::Medium] {
            // Client: publishes requests, subscribes to responses
            let client_pub = ctx.add_publication(8001);
            let client_sub = ctx.add_subscription(8002);

            // Server: subscribes to requests, publishes responses
            let server_sub = ctx.add_subscription(8001);
            let server_pub = ctx.add_publication(8002);

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

    pub fn bench_bidirectional(c: &mut Criterion) {
        let mut ctx = match BenchContext::new() {
            Some(c) => c,
            None => return,
        };

        // Bare
        bench_bidirectional_bare(c, &mut ctx);

        // Aerofoil
        bench_bidirectional_aerofoil(c, &mut ctx);
    }

    fn bench_bidirectional_aerofoil(c: &mut Criterion, ctx: &mut BenchContext) {
        let pub_a = ctx.add_publication(9001);
        let sub_a = ctx.add_subscription(9002);
        let mut publisher_a = AeronRsPublisher::new(pub_a);
        let mut subscriber_a = AeronRsSubscriber::new(sub_a);

        let pub_b = ctx.add_publication(9002);
        let sub_b = ctx.add_subscription(9001);
        let mut publisher_b = AeronRsPublisher::new(pub_b);
        let mut subscriber_b = AeronRsSubscriber::new(sub_b);

        generic::bench_bidirectional(
            c,
            &mut publisher_a,
            &mut subscriber_a,
            &mut publisher_b,
            &mut subscriber_b,
            "aeron-rs",
        );
    }

    fn bench_bidirectional_bare(c: &mut Criterion, ctx: &mut BenchContext) {
        let mut group = c.benchmark_group("aeron-rs/transceiver/bidirectional/bare");

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            // Side A: pub to B, sub from B
            let pub_a = ctx.add_publication(9001);
            let sub_a = ctx.add_subscription(9002);

            // Side B: pub to A, sub from A
            let pub_b = ctx.add_publication(9002);
            let sub_b = ctx.add_subscription(9001);

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

#[cfg(feature = "rusteron")]
use rusteron_bench::{
    bench_simultaneous_pub_sub as r_simul,
    bench_request_response as r_req_res,
    bench_bidirectional as r_bi
};

#[cfg(feature = "aeron-rs")]
use aeron_rs_bench::{
    bench_simultaneous_pub_sub as a_simul,
    bench_request_response as a_req_res,
    bench_bidirectional as a_bi
};

fn bench_wrapper(c: &mut Criterion) {
    #[cfg(feature = "rusteron")]
    {
        r_simul(c);
        r_req_res(c);
        r_bi(c);
    }
    #[cfg(feature = "aeron-rs")]
    {
        a_simul(c);
        a_req_res(c);
        a_bi(c);
    }
    #[cfg(not(any(feature = "rusteron", feature = "aeron-rs")))]
    eprintln!("Benchmarks require 'rusteron' or 'aeron-rs' feature.");
}

criterion_group!(benches, bench_wrapper);

criterion_main!(benches);
