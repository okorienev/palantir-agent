use criterion::{criterion_group, criterion_main, Criterion};
use palantir_agent_lib::metrics::histogram::builder::HistogramBuilder;
use palantir_agent_lib::metrics::traits::PrometheusMetric;
use rand::RngCore;

pub fn bench_track_random(c: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let val = rng.next_u32();
    let mut histogram = HistogramBuilder::named("some").finish();
    c.bench_function("Track random u32", |b| {
        b.iter(|| histogram.track(val as u64))
    });
}

pub fn bench_serialize_empty(c: &mut Criterion) {
    let histogram = HistogramBuilder::named("some").finish();
    c.bench_function("Serialize empty", |b| {
        b.iter(|| histogram.serialize_prometheus())
    });
}

pub fn bench_serialize_empty_10_tags(c: &mut Criterion) {
    let histogram = HistogramBuilder::named("some")
        .tag("key1", "val1")
        .tag("key2", "val2")
        .tag("key3", "val3")
        .tag("key4", "val4")
        .tag("key5", "val5")
        .tag("key6", "val6")
        .tag("key7", "val7")
        .tag("key8", "val8")
        .tag("key9", "val9")
        .tag("key10", "val10")
        .finish();

    c.bench_function("Serialize empty with 10 tags", |b| {
        b.iter(|| histogram.serialize_prometheus())
    });
}

pub fn bench_serialize_with_all_non_zero_buckets(c: &mut Criterion) {
    let mut histogram = HistogramBuilder::named("some").finish();

    let mut val = 128i64;
    for _ in 1..31 {
        histogram.track(val as u64);
        val <<= 1;
    }

    c.bench_function("Serialize with all non zero buckets", |b| {
        b.iter(|| histogram.serialize_prometheus())
    });
}

pub fn bench_serialize_non_zero_buckets_10_tags(c: &mut Criterion) {
    let mut histogram = HistogramBuilder::named("some")
        .tag("key1", "val1")
        .tag("key2", "val2")
        .tag("key3", "val3")
        .tag("key4", "val4")
        .tag("key5", "val5")
        .tag("key6", "val6")
        .tag("key7", "val7")
        .tag("key8", "val8")
        .tag("key9", "val9")
        .tag("key10", "val10")
        .finish();

    let mut val = 128i64;
    for _ in 1..31 {
        histogram.track(val as u64);
        val <<= 1;
    }
    c.bench_function("Serialize with all non zero buckets and 10 tags", |b| {
        b.iter(|| histogram.serialize_prometheus())
    });
}

criterion_group!(tracking, bench_track_random);
criterion_group!(
    serialization,
    bench_serialize_empty,
    bench_serialize_empty_10_tags,
    bench_serialize_with_all_non_zero_buckets,
    bench_serialize_non_zero_buckets_10_tags
);

criterion_main!(tracking, serialization);
