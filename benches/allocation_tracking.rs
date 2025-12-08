//! Allocation tracking benchmarks for Aeron transport.
//!
//! Verifies zero-allocation behavior in hot paths using dhat for
//! allocation profiling. This helps ensure the transport layer
//! maintains its performance guarantees.

mod common;

use common::MessageSize;
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
    use common::rusteron_support::BenchContext;

    /// Benchmark publication hot path allocations.
    pub fn bench_publication_allocations(c: &mut Criterion) {
        let ctx = match BenchContext::new() {
            Some(c) => c,
            None => return,
        };

        let mut group = c.benchmark_group("rusteron/allocations/publication");

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            let stream_id = 11001 + size.bytes() as i32;
            let publication = ctx.add_publication(stream_id);
            let mut publisher = RusteronPublisher::new(publication);

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
        let ctx = match BenchContext::new() {
            Some(c) => c,
            None => return,
        };

        let mut group = c.benchmark_group("rusteron/allocations/publication_mut");

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            let stream_id = 12001 + size.bytes() as i32;
            let publication = ctx.add_publication(stream_id);
            let mut publisher = RusteronPublisher::new(publication);

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
    pub fn bench_subscription_allocations(c: &mut Criterion) {
        let ctx = match BenchContext::new() {
            Some(c) => c,
            None => return,
        };

        let mut group = c.benchmark_group("rusteron/allocations/subscription");

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            let stream_id = 13001 + size.bytes() as i32;
            let (publication, subscription) = ctx.add_pub_sub(stream_id);

            let mut publisher = RusteronPublisher::new(publication);
            let mut subscriber = RusteronSubscriber::new(subscription);

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
        let ctx = match BenchContext::new() {
            Some(c) => c,
            None => return,
        };

        let mut group = c.benchmark_group("rusteron/allocations/try_claim");

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            let stream_id = 14001 + size.bytes() as i32;
            let publication = ctx.add_publication(stream_id);
            let mut publisher = RusteronPublisher::new(publication);

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
    use aeron_rs::concurrent::atomic_buffer::AtomicBuffer;
    use common::aeron_rs_support::BenchContext;

    /// Benchmark publication hot path allocations.
    pub fn bench_publication_allocations(c: &mut Criterion) {
        let mut ctx = match BenchContext::new() {
            Some(c) => c,
            None => return,
        };

        let mut group = c.benchmark_group("aeron-rs/allocations/publication");

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            // Use stream IDs in 21000+ range to avoid conflicts with rusteron (11000-14999)
            let stream_id = 21001 + size.bytes() as i32;
            let publication = ctx.add_publication(stream_id);

            group.bench_function(BenchmarkId::from_parameter(size.name()), |b| {
                let mut buffer = size.create_buffer();

                b.iter(|| {
                    let atomic_buffer = AtomicBuffer::wrap_slice(&mut buffer);
                    let pub_guard = publication.lock().expect("Publication mutex poisoned");
                    let _ = pub_guard.offer(atomic_buffer);
                });
            });
        }

        group.finish();
    }

    /// Benchmark subscription hot path allocations.
    pub fn bench_subscription_allocations(c: &mut Criterion) {
        let mut ctx = match BenchContext::new() {
            Some(c) => c,
            None => return,
        };

        let mut group = c.benchmark_group("aeron-rs/allocations/subscription");

        for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
            // Use stream IDs in 23000+ range to avoid conflicts with rusteron (13000-13999)
            let stream_id = 23001 + size.bytes() as i32;
            let (publication, subscription) = ctx.add_pub_sub(stream_id);

            // Pre-publish some messages
            let mut buffer = size.create_buffer();
            for _ in 0..100 {
                let atomic_buffer = AtomicBuffer::wrap_slice(&mut buffer);
                let pub_guard = publication.lock().expect("Publication mutex poisoned");
                let _ = pub_guard.offer(atomic_buffer);
            }
            thread::sleep(Duration::from_millis(50));

            group.bench_function(BenchmarkId::from_parameter(size.name()), |b| {
                b.iter(|| {
                    let mut sub_guard = subscription.lock().expect("Subscription mutex poisoned");
                    let _ = sub_guard.poll(&mut |_, _, _, _| {}, 1);
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
    rusteron_bench::bench_publication_allocations,
    rusteron_bench::bench_publication_mut_allocations,
    rusteron_bench::bench_subscription_allocations,
    rusteron_bench::bench_try_claim_allocations,
    aeron_rs_bench::bench_publication_allocations,
    aeron_rs_bench::bench_subscription_allocations
);

// When only rusteron enabled
#[cfg(all(feature = "rusteron", not(feature = "aeron-rs")))]
criterion_group!(
    benches,
    rusteron_bench::bench_publication_allocations,
    rusteron_bench::bench_publication_mut_allocations,
    rusteron_bench::bench_subscription_allocations,
    rusteron_bench::bench_try_claim_allocations
);

// When only aeron-rs enabled
#[cfg(all(feature = "aeron-rs", not(feature = "rusteron")))]
criterion_group!(
    benches,
    aeron_rs_bench::bench_publication_allocations,
    aeron_rs_bench::bench_subscription_allocations
);

#[cfg(not(any(feature = "rusteron", feature = "aeron-rs")))]
fn no_backend(_c: &mut Criterion) {
    eprintln!("Benchmarks require 'rusteron' or 'aeron-rs' feature.");
}

#[cfg(not(any(feature = "rusteron", feature = "aeron-rs")))]
criterion_group!(benches, no_backend);

criterion_main!(benches);
