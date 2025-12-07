//! Generic benchmark functions using Aeron traits.

#![allow(dead_code)]

use super::MessageSize;
use aerofoil::transport::{AeronPublisher, AeronSubscriber};
use criterion::{black_box, BenchmarkId, Criterion, Throughput};
use std::thread;
use std::time::Duration;

/// Benchmark aerofoil offer (with AeronPublisher trait abstraction).
pub fn bench_offer<P: AeronPublisher>(
    c: &mut Criterion,
    publisher: &mut P,
    name_prefix: &str,
) {
    let mut group = c.benchmark_group(format!("{}/offer/aerofoil", name_prefix));
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
                    let _ = black_box(publisher.offer(black_box(buf)));
                });
            },
        );
    }

    group.finish();
}

pub fn bench_offer_mut<P: AeronPublisher>(
    c: &mut Criterion,
    publisher: &mut P,
    name_prefix: &str,
) {
    let mut group = c.benchmark_group(format!("{}/offer_mut/aerofoil", name_prefix));
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

pub fn bench_try_claim<P: AeronPublisher>(
    c: &mut Criterion,
    publisher: &mut P,
    name_prefix: &str,
) {
    let mut group = c.benchmark_group(format!("{}/try_claim/aerofoil", name_prefix));
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

pub fn bench_poll<P: AeronPublisher, S: AeronSubscriber>(
    c: &mut Criterion,
    publisher: &mut P,
    subscriber: &mut S,
    name_prefix: &str,
) {
    let mut group = c.benchmark_group(format!("{}/poll/aerofoil", name_prefix));
    group.warm_up_time(Duration::from_millis(500));
    group.measurement_time(Duration::from_secs(1));
    group.sample_size(10);

    for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
        group.throughput(Throughput::Bytes(size.bytes() as u64));

        group.bench_function(BenchmarkId::from_parameter(size.name()), |b| {
            let buffer = size.create_buffer();

            b.iter(|| {
                let _ = publisher.offer(&buffer);
                let count = subscriber.poll(|_fragment| Ok(())).unwrap_or(0);
                black_box(count)
            });
        });
    }

    group.finish();
}

pub fn bench_simultaneous_pub_sub<P: AeronPublisher, S: AeronSubscriber>(
    c: &mut Criterion,
    publisher: &mut P,
    subscriber: &mut S,
    name_prefix: &str,
) {
    let mut group = c.benchmark_group(format!("{}/transceiver/simultaneous", name_prefix));

    for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
        group.throughput(Throughput::Bytes(size.bytes() as u64 * 2));

        group.bench_function(BenchmarkId::from_parameter(size.name()), |b| {
            let buffer = size.create_buffer();
            b.iter(|| {
                let pub_result = publisher.offer(&buffer);
                let poll_result = subscriber.poll(|_fragment| Ok(()));
                black_box((pub_result, poll_result))
            });
        });
    }
    group.finish();
}

pub fn bench_request_response<P: AeronPublisher, S: AeronSubscriber>(
    c: &mut Criterion,
    client_pub: &mut P,
    client_sub: &mut S,
    server_pub: &mut P,
    server_sub: &mut S,
    name_prefix: &str,
) {
    let mut group = c.benchmark_group(format!("{}/transceiver/roundtrip", name_prefix));

    // Warmup needed for reliable results
    thread::sleep(Duration::from_millis(100));

    for size in [MessageSize::Small, MessageSize::Medium] {
        group.throughput(Throughput::Elements(1));

        group.bench_function(BenchmarkId::from_parameter(size.name()), |b| {
            let request = size.create_buffer();
            let response = size.create_buffer();

            b.iter(|| {
                 // Client sends request
                 while client_pub.offer(&request).is_err() {
                     thread::yield_now();
                 }

                 // Server polls for request
                 let mut received_request = false;
                 for _ in 0..1000 {
                     let count = server_sub.poll(|_| Ok(())).unwrap_or(0);
                     if count > 0 {
                         received_request = true;
                         break;
                     }
                     thread::yield_now();
                 }

                 if received_request {
                     // Server sends response
                     while server_pub.offer(&response).is_err() {
                         thread::yield_now();
                     }

                     // Client polls for response
                     for _ in 0..1000 {
                         let count = client_sub.poll(|_| Ok(())).unwrap_or(0);
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

pub fn bench_bidirectional<P: AeronPublisher, S: AeronSubscriber>(
    c: &mut Criterion,
    pub_a: &mut P,
    sub_a: &mut S,
    pub_b: &mut P,
    sub_b: &mut S,
    name_prefix: &str,
) {
    let mut group = c.benchmark_group(format!("{}/transceiver/bidirectional", name_prefix));

    thread::sleep(Duration::from_millis(100));

    for size in [MessageSize::Small, MessageSize::Medium, MessageSize::Large] {
        group.throughput(Throughput::Bytes(size.bytes() as u64 * 2));

        group.bench_function(BenchmarkId::from_parameter(size.name()), |b| {
            let buffer_a = size.create_buffer();
            let buffer_b = size.create_buffer();

            b.iter(|| {
                // Both sides publish
                let _ = pub_a.offer(&buffer_a);
                let _ = pub_b.offer(&buffer_b);

                // Both sides poll
                let _ = sub_a.poll(|_| Ok(()));
                let _ = sub_b.poll(|_| Ok(()));
            });
        });
    }
    group.finish();
}
