//! Subscription throughput benchmarks for Aeron transport.
//!
//! Measures the throughput of receiving messages comparing:
//! - **bare**: Direct rusteron API calls (baseline)
//! - **aerofoil**: Calls through aerofoil's `AeronSubscriber` trait abstraction
//!
//! Note: Wingfoil nodes (`AeronSubscriberValueNode`, `AeronSubscriberValueRefNode`)
//! add minimal overhead (RefCell borrow + value storage) on top of the aerofoil layer.
//! The "aerofoil" benchmark effectively measures the node's core polling overhead.

mod common;

#[cfg(feature = "rusteron")]
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

    /// Setup raw rusteron pub/sub without aerofoil wrappers.
    fn setup_raw_pub_sub(
        aeron: &rusteron_client::Aeron,
        stream_id: i32,
    ) -> (
        rusteron_client::AeronPublication,
        rusteron_client::AeronSubscription,
    ) {
        let async_pub = aeron
            .async_add_publication(&CHANNEL.into_c_string(), stream_id)
            .expect("Failed to start publication");

        let publication = async_pub
            .poll_blocking(Duration::from_secs(5))
            .expect("Failed to complete publication");

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

        thread::sleep(Duration::from_millis(100));

        (publication, subscription)
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

        // Bare rusteron benchmarks (baseline)
        bench_poll_bare(c, &aeron);

        // Aerofoil trait abstraction benchmarks
        bench_poll_aerofoil(c, &aeron);

        // Additional benchmarks
        bench_poll_with_parsing(c, &aeron);
        bench_burst_throughput(c, &aeron);
    }

    /// Benchmark bare rusteron poll (baseline, no abstraction).
    fn bench_poll_bare(c: &mut Criterion, aeron: &rusteron_client::Aeron) {
        let mut group = c.benchmark_group("rusteron/poll/bare");
        group.warm_up_time(Duration::from_millis(500));
        group.measurement_time(Duration::from_secs(1));
        group.sample_size(10);

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            let stream_id = 5000 + size.bytes() as i32;

            let (publication, subscription) = setup_raw_pub_sub(aeron, stream_id);

            group.throughput(Throughput::Bytes(size.bytes() as u64));

            group.bench_function(BenchmarkId::from_parameter(size.name()), |b| {
                let buffer = size.create_buffer();

                b.iter(|| {
                    // Direct rusteron publish
                    let _ = black_box(
                        publication.offer::<rusteron_client::AeronReservedValueSupplierLogger>(
                            black_box(&buffer),
                            None,
                        ),
                    );

                    // Direct rusteron poll
                    let count = subscription
                        .poll_once(|_buffer, _header| {}, 1)
                        .unwrap_or(0);

                    black_box(count)
                });
            });
        }

        group.finish();
    }

    /// Benchmark aerofoil poll (with AeronSubscriber trait abstraction).
    fn bench_poll_aerofoil(c: &mut Criterion, aeron: &rusteron_client::Aeron) {
        let mut group = c.benchmark_group("rusteron/poll/aerofoil");
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
                    // Aerofoil publish
                    let _ = publisher.offer(&buffer);

                    // Aerofoil poll
                    let count = subscriber.poll(|_fragment| Ok(())).unwrap_or(0);

                    black_box(count)
                });
            });
        }

        group.finish();
    }

    /// Benchmark subscription with message parsing.
    fn bench_poll_with_parsing(c: &mut Criterion, aeron: &rusteron_client::Aeron) {
        let mut group = c.benchmark_group("rusteron/poll_parse");
        group.warm_up_time(Duration::from_millis(500));
        group.measurement_time(Duration::from_secs(2));
        group.sample_size(20);

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            let stream_id = 5101 + size.bytes() as i32;

            let (mut publisher, mut subscriber) = setup_pub_sub(aeron, stream_id);

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

    /// Benchmark burst subscription throughput.
    fn bench_burst_throughput(c: &mut Criterion, aeron: &rusteron_client::Aeron) {
        let mut group = c.benchmark_group("rusteron/poll_burst");
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
                    for _ in 0..burst_size {
                        while publisher.offer(&buffer).is_err() {
                            thread::yield_now();
                        }
                    }

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

#[cfg(all(feature = "aeron-rs", not(feature = "rusteron")))]
mod aeron_rs_bench {
    use super::*;
    use aeron_rs::aeron::Aeron;
    use aeron_rs::concurrent::atomic_buffer::AtomicBuffer;
    use aeron_rs::context::Context;
    use common::MediaDriverGuard;
    use std::ffi::CString;

    /// Run all aeron-rs subscription benchmarks.
    pub fn bench_all(c: &mut Criterion) {
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

        // Bare aeron-rs benchmarks (baseline)
        bench_poll_bare(c, &mut aeron);
    }

    /// Benchmark bare aeron-rs poll (baseline, no abstraction).
    fn bench_poll_bare(c: &mut Criterion, aeron: &mut Aeron) {
        let mut group = c.benchmark_group("aeron-rs/poll/bare");
        group.warm_up_time(Duration::from_millis(500));
        group.measurement_time(Duration::from_secs(1));
        group.sample_size(10);

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            let stream_id = 5000 + size.bytes() as i32;
            let channel = CString::new(CHANNEL).expect("Invalid channel");

            // Add publication
            let pub_reg_id = match aeron.add_publication(channel.clone(), stream_id) {
                Ok(id) => id,
                Err(e) => {
                    eprintln!("Failed to add publication: {:?}", e);
                    return;
                }
            };

            // Add subscription
            let sub_reg_id = match aeron.add_subscription(channel, stream_id) {
                Ok(id) => id,
                Err(e) => {
                    eprintln!("Failed to add subscription: {:?}", e);
                    return;
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

            group.throughput(Throughput::Bytes(size.bytes() as u64));

            group.bench_function(BenchmarkId::from_parameter(size.name()), |b| {
                let mut buffer = size.create_buffer();

                b.iter(|| {
                    // Direct aeron-rs publish
                    {
                        let atomic_buffer = AtomicBuffer::wrap_slice(&mut buffer);
                        let pub_guard = publication.lock().expect("Publication mutex poisoned");
                        let _ = black_box(pub_guard.offer(black_box(atomic_buffer)));
                    }

                    // Direct aeron-rs poll
                    let count = {
                        let mut sub_guard =
                            subscription.lock().expect("Subscription mutex poisoned");
                        sub_guard.poll(&mut |_buffer, _offset, _length, _header| {}, 1)
                    };

                    black_box(count)
                });
            });
        }

        group.finish();
    }
}

#[cfg(feature = "rusteron")]
criterion_group!(benches, rusteron_bench::bench_all);

#[cfg(all(feature = "aeron-rs", not(feature = "rusteron")))]
criterion_group!(benches, aeron_rs_bench::bench_all);

#[cfg(not(any(feature = "rusteron", feature = "aeron-rs")))]
fn no_backend(_c: &mut Criterion) {
    eprintln!("Benchmarks require 'rusteron' or 'aeron-rs' feature.");
}

#[cfg(not(any(feature = "rusteron", feature = "aeron-rs")))]
criterion_group!(benches, no_backend);

criterion_main!(benches);
