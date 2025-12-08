//! Publication latency benchmarks for Aeron transport.
//!
//! Measures the latency of publishing messages using the `offer` method
//! across different message sizes.
//!
//! Includes comparison between:
//! - **bare**: Direct API calls (baseline)
//! - **aerofoil**: Calls through aerofoil's `AeronPublisher` trait abstraction

mod common;

use aerofoil::transport::rusteron::RusteronPublisher;
use aerofoil::transport::AeronPublisher;
use common::rusteron_support::BenchContext;
use common::MessageSize;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Duration;

/// Run all benchmarks with a single shared media driver.
fn bench_all(c: &mut Criterion) {
    let ctx = match BenchContext::new() {
        Some(c) => c,
        None => return,
    };

    // Bare rusteron benchmarks (baseline)
    bench_offer_bare(c, &ctx);

    // Aerofoil trait abstraction benchmarks
    bench_offer_aerofoil(c, &ctx);
    bench_try_claim_aerofoil(c, &ctx);
}

/// Benchmark bare rusteron offer (baseline, no abstraction).
fn bench_offer_bare(c: &mut Criterion, ctx: &BenchContext) {
    let publication = ctx.add_publication(2000);

    let mut group = c.benchmark_group("offer/bare");
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
                        publication.offer::<rusteron_client::AeronReservedValueSupplierLogger>(
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
fn bench_offer_aerofoil(c: &mut Criterion, ctx: &BenchContext) {
    let publication = ctx.add_publication(2001);
    let mut publisher = RusteronPublisher::new(publication);

    let mut group = c.benchmark_group("offer/aerofoil");
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

fn bench_try_claim_aerofoil(c: &mut Criterion, ctx: &BenchContext) {
    let publication = ctx.add_publication(2003);
    let mut publisher = RusteronPublisher::new(publication);

    let mut group = c.benchmark_group("try_claim/aerofoil");
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

criterion_group!(benches, bench_all);
criterion_main!(benches);
