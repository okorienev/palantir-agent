use crate::lib::metrics::histogram::builder::HistogramBuilder;
use crate::lib::metrics::tag::Tag;
use palantir_types::*;

mod lib;

fn main() {
    let mut histogram = HistogramBuilder::named("histogram")
        .tag("key1", "val1")
        .tag("key2", "val2")
        .finish();

    histogram.track(1);
    histogram.track(2);
    println!("Hello, world!");
}
