//! Publication latency benchmarks for Aeron transport.
//!
//! Measures the latency of publishing messages using the `offer` and `offer_mut`
//! methods across different message sizes. Benchmarks are feature-gated to run
//! with either rusteron or aeron-rs backend.
//!
//! Includes comparison between:
//! - **bare**: Direct API calls (baseline)
//! - **aerofoil**: Calls through aerofoil's `AeronPublisher` trait abstraction

mod common;

use common::MessageSize;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Duration;

#[cfg(feature = "rusteron")]
mod rusteron_bench {
    use super::*;
    use aerofoil::transport::rusteron::RusteronPublisher;
    use aerofoil::transport::AeronPublisher;
    use common::rusteron_support::BenchContext;

    /// Run all rusteron benchmarks with a single shared media driver.
    pub fn bench_all(c: &mut Criterion) {
        let ctx = match BenchContext::new() {
            Some(c) => c,
            None => return,
        };

        // Bare rusteron benchmarks (baseline)
        bench_offer_bare(c, &ctx);

        // Aerofoil trait abstraction benchmarks
        bench_offer_aerofoil(c, &ctx);
        bench_offer_mut_aerofoil(c, &ctx);
        bench_try_claim_aerofoil(c, &ctx);
    }

    /// Benchmark bare rusteron offer (baseline, no abstraction).
    fn bench_offer_bare(c: &mut Criterion, ctx: &BenchContext) {
        let publication = ctx.add_publication(2000);

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
    fn bench_offer_aerofoil(c: &mut Criterion, ctx: &BenchContext) {
        let publication = ctx.add_publication(2001);
        let mut publisher = RusteronPublisher::new(publication);

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

    fn bench_offer_mut_aerofoil(c: &mut Criterion, ctx: &BenchContext) {
        let publication = ctx.add_publication(2002);
        let mut publisher = RusteronPublisher::new(publication);

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

    fn bench_try_claim_aerofoil(c: &mut Criterion, ctx: &BenchContext) {
        let publication = ctx.add_publication(2003);
        let mut publisher = RusteronPublisher::new(publication);

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

#[cfg(feature = "aeron-rs")]
mod aeron_rs_bench {
    use super::*;
    use aeron_rs::concurrent::atomic_buffer::AtomicBuffer;
    use common::aeron_rs_support::BenchContext;

    /// Run all aeron-rs benchmarks.
    pub fn bench_all(c: &mut Criterion) {
        let mut ctx = match BenchContext::new() {
            Some(c) => c,
            None => return,
        };

        // Bare aeron-rs benchmarks (baseline)
        bench_offer_bare(c, &mut ctx);
    }

    /// Benchmark bare aeron-rs offer (baseline, no abstraction).
    fn bench_offer_bare(c: &mut Criterion, ctx: &mut BenchContext) {
        // Use stream IDs in 3000+ range to avoid conflicts with rusteron (1000-1999)
        let publication = ctx.add_publication(3000);

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

// When both features enabled, run both benchmark suites
#[cfg(all(feature = "rusteron", feature = "aeron-rs"))]
criterion_group!(benches, rusteron_bench::bench_all, aeron_rs_bench::bench_all);

// When only rusteron enabled
#[cfg(all(feature = "rusteron", not(feature = "aeron-rs")))]
criterion_group!(benches, rusteron_bench::bench_all);

// When only aeron-rs enabled
#[cfg(all(feature = "aeron-rs", not(feature = "rusteron")))]
criterion_group!(benches, aeron_rs_bench::bench_all);

#[cfg(not(any(feature = "rusteron", feature = "aeron-rs")))]
fn no_backend(_c: &mut Criterion) {
    eprintln!("Benchmarks require 'rusteron' or 'aeron-rs' feature.");
}

#[cfg(not(any(feature = "rusteron", feature = "aeron-rs")))]
criterion_group!(benches, no_backend);

criterion_main!(benches);
