//! Allocation tracking benchmarks for Aeron transport.
//!
//! Verifies zero-allocation behavior in hot paths using dhat for
//! allocation profiling. This helps ensure the transport layer
//! maintains its performance guarantees.

mod common;

use common::{MessageSize, CHANNEL};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::thread;
use std::time::Duration;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[cfg(feature = "rusteron")]
mod rusteron_bench {
    use super::*;
    use aerofoil::transport::rusteron::{RusteronPublisher, RusteronSubscriber};
    use aerofoil::transport::{AeronPublisher, AeronSubscriber};
    use common::MediaDriverGuard;
    use rusteron_client::IntoCString;

    fn setup_pub_sub(
        stream_id: i32,
    ) -> Option<(MediaDriverGuard, RusteronPublisher, RusteronSubscriber)> {
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

        Some((driver, publisher, subscriber))
    }

    /// Benchmark publication hot path allocations.
    ///
    /// Measures allocations during the offer() call to verify
    /// zero-allocation behavior in the publication path.
    pub fn bench_publication_allocations(c: &mut Criterion) {
        let mut group = c.benchmark_group("rusteron/allocations/publication");

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            let stream_id = 11001 + size.bytes() as i32;

            let (_driver, mut publisher, _subscriber) = match setup_pub_sub(stream_id) {
                Some(s) => s,
                None => return,
            };

            let buffer = size.create_buffer();

            group.bench_function(BenchmarkId::from_parameter(size.name()), |b| {
                b.iter(|| {
                    // Measure allocations during offer
                    let _ = publisher.offer(&buffer);
                });
            });
        }

        group.finish();
    }

    /// Benchmark publication hot path allocations with mutable buffer.
    pub fn bench_publication_mut_allocations(c: &mut Criterion) {
        let mut group = c.benchmark_group("rusteron/allocations/publication_mut");

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            let stream_id = 12001 + size.bytes() as i32;

            let (_driver, mut publisher, _subscriber) = match setup_pub_sub(stream_id) {
                Some(s) => s,
                None => return,
            };

            group.bench_function(BenchmarkId::from_parameter(size.name()), |b| {
                let mut buffer = size.create_buffer();
                b.iter(|| {
                    let _ = publisher.offer_mut(&mut buffer);
                });
            });
        }

        group.finish();
    }

    /// Benchmark subscription hot path allocations.
    ///
    /// Measures allocations during the poll() call to verify
    /// zero-allocation behavior in the subscription path.
    pub fn bench_subscription_allocations(c: &mut Criterion) {
        let mut group = c.benchmark_group("rusteron/allocations/subscription");

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            let stream_id = 13001 + size.bytes() as i32;

            let (_driver, mut publisher, mut subscriber) = match setup_pub_sub(stream_id) {
                Some(s) => s,
                None => return,
            };

            // Pre-publish some messages
            let buffer = size.create_buffer();
            for _ in 0..100 {
                let _ = publisher.offer(&buffer);
            }
            thread::sleep(Duration::from_millis(50));

            group.bench_function(BenchmarkId::from_parameter(size.name()), |b| {
                b.iter(|| {
                    // Measure allocations during poll
                    let _ = subscriber.poll(|_fragment| Ok(()));
                });
            });
        }

        group.finish();
    }

    /// Benchmark try_claim hot path allocations.
    pub fn bench_try_claim_allocations(c: &mut Criterion) {
        let mut group = c.benchmark_group("rusteron/allocations/try_claim");

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            let stream_id = 14001 + size.bytes() as i32;

            let (_driver, mut publisher, _subscriber) = match setup_pub_sub(stream_id) {
                Some(s) => s,
                None => return,
            };

            let data = size.create_buffer();

            group.bench_function(BenchmarkId::from_parameter(size.name()), |b| {
                b.iter(|| {
                    if let Ok(mut claim) = publisher.try_claim(data.len()) {
                        claim.copy_from_slice(&data);
                    }
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
    use aerofoil::transport::{AeronPublisher, AeronSubscriber};
    use aeron_rs::client::Client;
    use aeron_rs::context::Context;

    /// Benchmark publication hot path allocations for aeron-rs.
    pub fn bench_publication_allocations(c: &mut Criterion) {
        let context = Context::new();
        let mut client = Client::connect(context).expect("Failed to connect to media driver");

        let mut group = c.benchmark_group("aeron-rs/allocations/publication");

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            let stream_id = 15001 + size.bytes() as i32;

            let publication = client
                .add_publication(CHANNEL, stream_id)
                .expect("Failed to add publication");

            let mut publisher = AeronRsPublisher::new(publication);

            thread::sleep(Duration::from_millis(100));

            let buffer = size.create_buffer();

            group.bench_function(BenchmarkId::from_parameter(size.name()), |b| {
                b.iter(|| {
                    let _ = publisher.offer(&buffer);
                });
            });
        }

        group.finish();
    }

    /// Benchmark publication hot path allocations with mutable buffer for aeron-rs.
    ///
    /// This is particularly important for aeron-rs since offer_mut avoids
    /// the internal copy that offer requires.
    pub fn bench_publication_mut_allocations(c: &mut Criterion) {
        let context = Context::new();
        let mut client = Client::connect(context).expect("Failed to connect to media driver");

        let mut group = c.benchmark_group("aeron-rs/allocations/publication_mut");

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            let stream_id = 16001 + size.bytes() as i32;

            let publication = client
                .add_publication(CHANNEL, stream_id)
                .expect("Failed to add publication");

            let mut publisher = AeronRsPublisher::new(publication);

            thread::sleep(Duration::from_millis(100));

            group.bench_function(BenchmarkId::from_parameter(size.name()), |b| {
                let mut buffer = size.create_buffer();
                b.iter(|| {
                    let _ = publisher.offer_mut(&mut buffer);
                });
            });
        }

        group.finish();
    }

    /// Benchmark subscription hot path allocations for aeron-rs.
    pub fn bench_subscription_allocations(c: &mut Criterion) {
        let context = Context::new();
        let mut client = Client::connect(context).expect("Failed to connect to media driver");

        let mut group = c.benchmark_group("aeron-rs/allocations/subscription");

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            let stream_id = 17001 + size.bytes() as i32;

            let publication = client
                .add_publication(CHANNEL, stream_id)
                .expect("Failed to add publication");
            let subscription = client
                .add_subscription(CHANNEL, stream_id)
                .expect("Failed to add subscription");

            let mut publisher = AeronRsPublisher::new(publication);
            let mut subscriber = AeronRsSubscriber::new(subscription);

            thread::sleep(Duration::from_millis(100));

            // Pre-publish some messages
            let buffer = size.create_buffer();
            for _ in 0..100 {
                let _ = publisher.offer(&buffer);
            }
            thread::sleep(Duration::from_millis(50));

            group.bench_function(BenchmarkId::from_parameter(size.name()), |b| {
                b.iter(|| {
                    let _ = subscriber.poll(|_fragment| Ok(()));
                });
            });
        }

        group.finish();
    }
}

#[cfg(feature = "rusteron")]
criterion_group!(
    benches,
    rusteron_bench::bench_publication_allocations,
    rusteron_bench::bench_publication_mut_allocations,
    rusteron_bench::bench_subscription_allocations,
    rusteron_bench::bench_try_claim_allocations
);

#[cfg(all(feature = "aeron-rs", not(feature = "rusteron")))]
criterion_group!(
    benches,
    aeron_rs_bench::bench_publication_allocations,
    aeron_rs_bench::bench_publication_mut_allocations,
    aeron_rs_bench::bench_subscription_allocations
);

#[cfg(not(any(feature = "rusteron", feature = "aeron-rs")))]
fn no_backend(_c: &mut Criterion) {
    eprintln!("No Aeron backend enabled. Enable 'rusteron' or 'aeron-rs' feature.");
}

#[cfg(not(any(feature = "rusteron", feature = "aeron-rs")))]
criterion_group!(benches, no_backend);

criterion_main!(benches);
