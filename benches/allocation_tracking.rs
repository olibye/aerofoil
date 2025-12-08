//! Allocation tracking benchmarks for Aeron transport.
//!
//! Verifies zero-allocation behavior in hot paths using dhat for
//! allocation profiling. This helps ensure the transport layer
//! maintains its performance guarantees.

mod common;

use aerofoil::transport::rusteron::{RusteronPublisher, RusteronSubscriber};
use aerofoil::transport::{AeronPublisher, AeronSubscriber};
use common::rusteron_support::BenchContext;
use common::MessageSize;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::thread;
use std::time::Duration;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

/// Benchmark publication hot path allocations.
fn bench_publication_allocations(c: &mut Criterion) {
    let ctx = match BenchContext::new() {
        Some(c) => c,
        None => return,
    };

    let mut group = c.benchmark_group("allocations/publication");

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
fn bench_publication_mut_allocations(c: &mut Criterion) {
    let ctx = match BenchContext::new() {
        Some(c) => c,
        None => return,
    };

    let mut group = c.benchmark_group("allocations/publication_mut");

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
fn bench_subscription_allocations(c: &mut Criterion) {
    let ctx = match BenchContext::new() {
        Some(c) => c,
        None => return,
    };

    let mut group = c.benchmark_group("allocations/subscription");

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
fn bench_try_claim_allocations(c: &mut Criterion) {
    let ctx = match BenchContext::new() {
        Some(c) => c,
        None => return,
    };

    let mut group = c.benchmark_group("allocations/try_claim");

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

criterion_group!(
    benches,
    bench_publication_allocations,
    bench_publication_mut_allocations,
    bench_subscription_allocations,
    bench_try_claim_allocations
);
criterion_main!(benches);
