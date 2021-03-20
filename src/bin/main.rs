use palantir_agent_lib::metrics::histogram::builder::HistogramBuilder;
use palantir_agent_lib::metrics::traits::PrometheusMetric;
use palantir_types::*;

fn main() {
    let mut histogram = HistogramBuilder::named("histogram")
        .tag("key1", "val1")
        .tag("key2", "val2")
        .finish();

    let mut val = 128i64;
    for _ in 1..31 {
        histogram.track(val as u64);
        val <<= 1;
    }
    for i in histogram.serialize_prometheus() {
        println!("{}", i)
    }
}
