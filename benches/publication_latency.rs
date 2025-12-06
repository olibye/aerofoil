//! Publication latency benchmarks for Aeron transport.
//!
//! Measures the latency of publishing messages using the `offer` and `offer_mut`
//! methods across different message sizes. Benchmarks are feature-gated to run
//! with either rusteron or aeron-rs backend.
//!
//! Includes comparison between:
//! - **bare**: Direct rusteron API calls (baseline)
//! - **aerofoil**: Calls through aerofoil's `AeronPublisher` trait abstraction

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
    use aerofoil::transport::rusteron::RusteronPublisher;
    use common::MediaDriverGuard;
    use rusteron_client::IntoCString;

    /// Run all rusteron benchmarks with a single shared media driver.
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
        bench_offer_bare(c, &aeron);

        // Aerofoil trait abstraction benchmarks
        bench_offer_aerofoil(c, &aeron);
        bench_offer_mut_aerofoil(c, &aeron);
        bench_try_claim_aerofoil(c, &aeron);
    }

    /// Benchmark bare rusteron offer (baseline, no abstraction).
    fn bench_offer_bare(c: &mut Criterion, aeron: &rusteron_client::Aeron) {
        let stream_id = 2000;
        let async_pub = aeron
            .async_add_publication(&CHANNEL.into_c_string(), stream_id)
            .expect("Failed to start publication");

        let publication = async_pub
            .poll_blocking(Duration::from_secs(5))
            .expect("Failed to complete publication");

        thread::sleep(Duration::from_millis(100));

        let mut group = c.benchmark_group("rusteron/offer/bare");
        group.warm_up_time(Duration::from_millis(500));
        group.measurement_time(Duration::from_secs(2));
        group.sample_size(20);

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            let buffer = size.create_buffer();
            group.throughput(Throughput::Bytes(size.bytes() as u64));

            group.bench_with_input(
                BenchmarkId::from_parameter(size.name()),
                &buffer,
                |b, buf| {
                    b.iter(|| {
                        // Direct rusteron call - no trait abstraction
                        let _ = black_box(
                            publication
                                .offer::<rusteron_client::AeronReservedValueSupplierLogger>(
                                    black_box(buf),
                                    None,
                                ),
                        );
                    });
                },
            );
        }

        group.finish();
    }

    /// Benchmark aerofoil offer (with AeronPublisher trait abstraction).
    fn bench_offer_aerofoil(c: &mut Criterion, aeron: &rusteron_client::Aeron) {
        let stream_id = 2001;
        let async_pub = aeron
            .async_add_publication(&CHANNEL.into_c_string(), stream_id)
            .expect("Failed to start publication");

        let publication = async_pub
            .poll_blocking(Duration::from_secs(5))
            .expect("Failed to complete publication");

        let mut publisher = RusteronPublisher::new(publication);

        thread::sleep(Duration::from_millis(100));

        let mut group = c.benchmark_group("rusteron/offer/aerofoil");
        group.warm_up_time(Duration::from_millis(500));
        group.measurement_time(Duration::from_secs(2));
        group.sample_size(20);

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            let buffer = size.create_buffer();
            group.throughput(Throughput::Bytes(size.bytes() as u64));

            group.bench_with_input(
                BenchmarkId::from_parameter(size.name()),
                &buffer,
                |b, buf| {
                    b.iter(|| {
                        // Through aerofoil's AeronPublisher trait
                        let _ = black_box(publisher.offer(black_box(buf)));
                    });
                },
            );
        }

        group.finish();
    }

    fn bench_offer_mut_aerofoil(c: &mut Criterion, aeron: &rusteron_client::Aeron) {
        let stream_id = 2002;
        let async_pub = aeron
            .async_add_publication(&CHANNEL.into_c_string(), stream_id)
            .expect("Failed to start publication");

        let publication = async_pub
            .poll_blocking(Duration::from_secs(5))
            .expect("Failed to complete publication");

        let mut publisher = RusteronPublisher::new(publication);

        thread::sleep(Duration::from_millis(100));

        let mut group = c.benchmark_group("rusteron/offer_mut/aerofoil");
        group.warm_up_time(Duration::from_millis(500));
        group.measurement_time(Duration::from_secs(2));
        group.sample_size(20);

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            group.throughput(Throughput::Bytes(size.bytes() as u64));

            group.bench_function(BenchmarkId::from_parameter(size.name()), |b| {
                let mut buffer = size.create_buffer();
                b.iter(|| {
                    let _ = black_box(publisher.offer_mut(black_box(&mut buffer)));
                });
            });
        }

        group.finish();
    }

    fn bench_try_claim_aerofoil(c: &mut Criterion, aeron: &rusteron_client::Aeron) {
        let stream_id = 2003;
        let async_pub = aeron
            .async_add_publication(&CHANNEL.into_c_string(), stream_id)
            .expect("Failed to start publication");

        let publication = async_pub
            .poll_blocking(Duration::from_secs(5))
            .expect("Failed to complete publication");

        let mut publisher = RusteronPublisher::new(publication);

        thread::sleep(Duration::from_millis(100));

        let mut group = c.benchmark_group("rusteron/try_claim/aerofoil");
        group.warm_up_time(Duration::from_millis(500));
        group.measurement_time(Duration::from_secs(2));
        group.sample_size(20);

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            let data = size.create_buffer();
            group.throughput(Throughput::Bytes(size.bytes() as u64));

            group.bench_with_input(
                BenchmarkId::from_parameter(size.name()),
                &data,
                |b, data| {
                    b.iter(|| {
                        if let Ok(mut claim) = publisher.try_claim(data.len()) {
                            claim.copy_from_slice(data);
                        }
                    });
                },
            );
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

    /// Run all aeron-rs benchmarks.
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
        bench_offer_bare(c, &mut aeron);
    }

    /// Benchmark bare aeron-rs offer (baseline, no abstraction).
    fn bench_offer_bare(c: &mut Criterion, aeron: &mut Aeron) {
        let stream_id = 2000;
        let channel = CString::new(CHANNEL).expect("Invalid channel");

        let registration_id = match aeron.add_publication(channel, stream_id) {
            Ok(id) => id,
            Err(e) => {
                eprintln!("Failed to add publication: {:?}", e);
                return;
            }
        };

        // Poll until publication is ready
        let publication = loop {
            match aeron.find_publication(registration_id) {
                Ok(pub_arc) => break pub_arc,
                Err(_) => {
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
        };

        thread::sleep(Duration::from_millis(100));

        let mut group = c.benchmark_group("aeron-rs/offer/bare");
        group.warm_up_time(Duration::from_millis(500));
        group.measurement_time(Duration::from_secs(2));
        group.sample_size(20);

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            let mut buffer = size.create_buffer();
            group.throughput(Throughput::Bytes(size.bytes() as u64));

            group.bench_with_input(
                BenchmarkId::from_parameter(size.name()),
                &size,
                |b, _| {
                    b.iter(|| {
                        // Direct aeron-rs call
                        let atomic_buffer = AtomicBuffer::wrap_slice(&mut buffer);
                        let pub_guard = publication.lock().expect("Publication mutex poisoned");
                        let _ = black_box(pub_guard.offer(black_box(atomic_buffer)));
                    });
                },
            );
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
