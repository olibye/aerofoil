//! Combined pub/sub (transceiver) benchmarks for Aeron transport.
//!
//! Measures performance when publishing and subscribing concurrently,
//! including request/response roundtrip latency and bidirectional exchange.

mod common;

#[cfg(feature = "rusteron")]
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

#[cfg(all(feature = "aeron-rs", not(feature = "rusteron")))]
mod aeron_rs_bench {
    use super::*;
    use aeron_rs::aeron::Aeron;
    use aeron_rs::concurrent::atomic_buffer::AtomicBuffer;
    use aeron_rs::context::Context;
    use aeron_rs::publication::Publication;
    use aeron_rs::subscription::Subscription;
    use common::MediaDriverGuard;
    use std::ffi::CString;
    use std::sync::{Arc, Mutex};

    fn setup_transceiver(
        aeron: &mut Aeron,
        pub_stream: i32,
        sub_stream: i32,
    ) -> Option<(Arc<Mutex<Publication>>, Arc<Mutex<Subscription>>)> {
        let channel = CString::new(CHANNEL).expect("Invalid channel");

        // Add publication
        let pub_reg_id = match aeron.add_publication(channel.clone(), pub_stream) {
            Ok(id) => id,
            Err(e) => {
                eprintln!("Failed to add publication: {:?}", e);
                return None;
            }
        };

        // Add subscription
        let sub_reg_id = match aeron.add_subscription(channel, sub_stream) {
            Ok(id) => id,
            Err(e) => {
                eprintln!("Failed to add subscription: {:?}", e);
                return None;
            }
        };

        // Poll until ready
        let publication = loop {
            match aeron.find_publication(pub_reg_id) {
                Ok(pub_arc) => break pub_arc,
                Err(_) => std::thread::sleep(Duration::from_millis(10)),
            }
        };

        let subscription = loop {
            match aeron.find_subscription(sub_reg_id) {
                Ok(sub_arc) => break sub_arc,
                Err(_) => std::thread::sleep(Duration::from_millis(10)),
            }
        };

        thread::sleep(Duration::from_millis(100));

        Some((publication, subscription))
    }

    /// Benchmark simultaneous publish/subscribe on different streams.
    pub fn bench_simultaneous_pub_sub(c: &mut Criterion) {
        let _driver = match MediaDriverGuard::start() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Skipping benchmark: {}", e);
                return;
            }
        };

        let context = Context::new();
        let mut aeron = match Aeron::new(context) {
            Ok(a) => a,
            Err(e) => {
                eprintln!(
                    "Failed to connect to Aeron media driver: {:?}\n\
                     Ensure an external media driver is running:\n\
                     java -cp aeron-all.jar io.aeron.driver.MediaDriver",
                    e
                );
                return;
            }
        };

        let mut group = c.benchmark_group("aeron-rs/transceiver/simultaneous");

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            let pub_stream = 7001;
            let sub_stream = 7002;

            let (publication, subscription) = match setup_transceiver(&mut aeron, pub_stream, sub_stream) {
                Some(s) => s,
                None => return,
            };

            group.throughput(Throughput::Bytes(size.bytes() as u64 * 2)); // pub + sub

            group.bench_function(BenchmarkId::from_parameter(size.name()), |b| {
                let mut buffer = size.create_buffer();

                b.iter(|| {
                    // Publish
                    let pub_result = {
                        let atomic_buffer = AtomicBuffer::wrap_slice(&mut buffer);
                        let pub_guard = publication.lock().expect("Publication mutex poisoned");
                        pub_guard.offer(atomic_buffer)
                    };

                    // Poll
                    let poll_result = {
                        let mut sub_guard = subscription.lock().expect("Subscription mutex poisoned");
                        sub_guard.poll(&mut |_buffer, _offset, _length, _header| {}, 1)
                    };

                    black_box((pub_result, poll_result))
                });
            });
        }

        group.finish();
    }

    /// Benchmark request/response roundtrip latency.
    pub fn bench_request_response(c: &mut Criterion) {
        let _driver = match MediaDriverGuard::start() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Skipping benchmark: {}", e);
                return;
            }
        };

        let context = Context::new();
        let mut aeron = match Aeron::new(context) {
            Ok(a) => a,
            Err(e) => {
                eprintln!(
                    "Failed to connect to Aeron media driver: {:?}\n\
                     Ensure an external media driver is running:\n\
                     java -cp aeron-all.jar io.aeron.driver.MediaDriver",
                    e
                );
                return;
            }
        };

        let mut group = c.benchmark_group("aeron-rs/transceiver/roundtrip");

        for size in [MessageSize::Small, MessageSize::Medium] {
            let request_stream = 8001;
            let response_stream = 8002;
            let channel = CString::new(CHANNEL).expect("Invalid channel");

            // Client: publishes requests, subscribes to responses
            let client_pub_id = aeron.add_publication(channel.clone(), request_stream).expect("Failed");
            let client_sub_id = aeron.add_subscription(channel.clone(), response_stream).expect("Failed");

            // Server: subscribes to requests, publishes responses
            let server_sub_id = aeron.add_subscription(channel.clone(), request_stream).expect("Failed");
            let server_pub_id = aeron.add_publication(channel, response_stream).expect("Failed");

            // Wait for all to be ready
            let client_pub = loop {
                match aeron.find_publication(client_pub_id) {
                    Ok(p) => break p,
                    Err(_) => std::thread::sleep(Duration::from_millis(10)),
                }
            };
            let client_sub = loop {
                match aeron.find_subscription(client_sub_id) {
                    Ok(s) => break s,
                    Err(_) => std::thread::sleep(Duration::from_millis(10)),
                }
            };
            let server_sub = loop {
                match aeron.find_subscription(server_sub_id) {
                    Ok(s) => break s,
                    Err(_) => std::thread::sleep(Duration::from_millis(10)),
                }
            };
            let server_pub = loop {
                match aeron.find_publication(server_pub_id) {
                    Ok(p) => break p,
                    Err(_) => std::thread::sleep(Duration::from_millis(10)),
                }
            };

            thread::sleep(Duration::from_millis(200));

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
        let _driver = match MediaDriverGuard::start() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Skipping benchmark: {}", e);
                return;
            }
        };

        let context = Context::new();
        let mut aeron = match Aeron::new(context) {
            Ok(a) => a,
            Err(e) => {
                eprintln!(
                    "Failed to connect to Aeron media driver: {:?}\n\
                     Ensure an external media driver is running:\n\
                     java -cp aeron-all.jar io.aeron.driver.MediaDriver",
                    e
                );
                return;
            }
        };

        let mut group = c.benchmark_group("aeron-rs/transceiver/bidirectional");

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            let stream_a_to_b = 9001;
            let stream_b_to_a = 9002;
            let channel = CString::new(CHANNEL).expect("Invalid channel");

            // Side A: pub to B, sub from B
            let pub_a_id = aeron.add_publication(channel.clone(), stream_a_to_b).expect("Failed");
            let sub_a_id = aeron.add_subscription(channel.clone(), stream_b_to_a).expect("Failed");

            // Side B: pub to A, sub from A
            let pub_b_id = aeron.add_publication(channel.clone(), stream_b_to_a).expect("Failed");
            let sub_b_id = aeron.add_subscription(channel, stream_a_to_b).expect("Failed");

            // Wait for all to be ready
            let pub_a = loop {
                match aeron.find_publication(pub_a_id) {
                    Ok(p) => break p,
                    Err(_) => std::thread::sleep(Duration::from_millis(10)),
                }
            };
            let sub_a = loop {
                match aeron.find_subscription(sub_a_id) {
                    Ok(s) => break s,
                    Err(_) => std::thread::sleep(Duration::from_millis(10)),
                }
            };
            let pub_b = loop {
                match aeron.find_publication(pub_b_id) {
                    Ok(p) => break p,
                    Err(_) => std::thread::sleep(Duration::from_millis(10)),
                }
            };
            let sub_b = loop {
                match aeron.find_subscription(sub_b_id) {
                    Ok(s) => break s,
                    Err(_) => std::thread::sleep(Duration::from_millis(10)),
                }
            };

            thread::sleep(Duration::from_millis(200));

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
criterion_group!(
    benches,
    rusteron_bench::bench_simultaneous_pub_sub,
    rusteron_bench::bench_request_response,
    rusteron_bench::bench_bidirectional
);

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
