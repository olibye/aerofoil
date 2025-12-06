//! Publication latency benchmarks for Aeron transport.
//!
//! Measures the latency of publishing messages using the `offer` and `offer_mut`
//! methods across different message sizes. Benchmarks are feature-gated to run
//! with either rusteron or aeron-rs backend.

mod common;

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

    pub fn bench_offer(c: &mut Criterion) {
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

        let stream_id = 2001;
        let async_pub = aeron
            .async_add_publication(&CHANNEL.into_c_string(), stream_id)
            .expect("Failed to start publication");

        let publication = async_pub
            .poll_blocking(Duration::from_secs(5))
            .expect("Failed to complete publication");

        let mut publisher = RusteronPublisher::new(publication);

        thread::sleep(Duration::from_millis(100));

        let mut group = c.benchmark_group("rusteron/offer");

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            let buffer = size.create_buffer();
            group.throughput(Throughput::Bytes(size.bytes() as u64));

            group.bench_with_input(
                BenchmarkId::from_parameter(size.name()),
                &buffer,
                |b, buf| {
                    b.iter(|| {
                        let _ = black_box(publisher.offer(black_box(buf)));
                    });
                },
            );
        }

        group.finish();
    }

    pub fn bench_offer_mut(c: &mut Criterion) {
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

        let stream_id = 2002;
        let async_pub = aeron
            .async_add_publication(&CHANNEL.into_c_string(), stream_id)
            .expect("Failed to start publication");

        let publication = async_pub
            .poll_blocking(Duration::from_secs(5))
            .expect("Failed to complete publication");

        let mut publisher = RusteronPublisher::new(publication);

        thread::sleep(Duration::from_millis(100));

        let mut group = c.benchmark_group("rusteron/offer_mut");

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

    pub fn bench_try_claim(c: &mut Criterion) {
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

        let stream_id = 2003;
        let async_pub = aeron
            .async_add_publication(&CHANNEL.into_c_string(), stream_id)
            .expect("Failed to start publication");

        let publication = async_pub
            .poll_blocking(Duration::from_secs(5))
            .expect("Failed to complete publication");

        let mut publisher = RusteronPublisher::new(publication);

        thread::sleep(Duration::from_millis(100));

        let mut group = c.benchmark_group("rusteron/try_claim");

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

#[cfg(feature = "aeron-rs")]
mod aeron_rs_bench {
    use super::*;
    use aerofoil::transport::aeron_rs::AeronRsPublisher;
    use aeron_rs::client::Client;
    use aeron_rs::context::Context;

    pub fn bench_offer(c: &mut Criterion) {
        let context = Context::new();
        let mut client = Client::connect(context).expect("Failed to connect to media driver");

        let stream_id = 3001;
        let publication = client
            .add_publication(CHANNEL, stream_id)
            .expect("Failed to add publication");

        let mut publisher = AeronRsPublisher::new(publication);

        thread::sleep(Duration::from_millis(100));

        let mut group = c.benchmark_group("aeron-rs/offer");

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            let buffer = size.create_buffer();
            group.throughput(Throughput::Bytes(size.bytes() as u64));

            group.bench_with_input(
                BenchmarkId::from_parameter(size.name()),
                &buffer,
                |b, buf| {
                    b.iter(|| {
                        let _ = black_box(publisher.offer(black_box(buf)));
                    });
                },
            );
        }

        group.finish();
    }

    pub fn bench_offer_mut(c: &mut Criterion) {
        let context = Context::new();
        let mut client = Client::connect(context).expect("Failed to connect to media driver");

        let stream_id = 3002;
        let publication = client
            .add_publication(CHANNEL, stream_id)
            .expect("Failed to add publication");

        let mut publisher = AeronRsPublisher::new(publication);

        thread::sleep(Duration::from_millis(100));

        let mut group = c.benchmark_group("aeron-rs/offer_mut");

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

    pub fn bench_try_claim(c: &mut Criterion) {
        let context = Context::new();
        let mut client = Client::connect(context).expect("Failed to connect to media driver");

        let stream_id = 3003;
        let publication = client
            .add_publication(CHANNEL, stream_id)
            .expect("Failed to add publication");

        let mut publisher = AeronRsPublisher::new(publication);

        thread::sleep(Duration::from_millis(100));

        let mut group = c.benchmark_group("aeron-rs/try_claim");

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

#[cfg(feature = "rusteron")]
criterion_group!(
    benches,
    rusteron_bench::bench_offer,
    rusteron_bench::bench_offer_mut,
    rusteron_bench::bench_try_claim
);

#[cfg(all(feature = "aeron-rs", not(feature = "rusteron")))]
criterion_group!(
    benches,
    aeron_rs_bench::bench_offer,
    aeron_rs_bench::bench_offer_mut,
    aeron_rs_bench::bench_try_claim
);

#[cfg(not(any(feature = "rusteron", feature = "aeron-rs")))]
fn no_backend(_c: &mut Criterion) {
    eprintln!("No Aeron backend enabled. Enable 'rusteron' or 'aeron-rs' feature.");
}

#[cfg(not(any(feature = "rusteron", feature = "aeron-rs")))]
criterion_group!(benches, no_backend);

criterion_main!(benches);
