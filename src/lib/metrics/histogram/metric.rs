use crate::lib::metrics::tag::Tag;
use crate::lib::metrics::traits::PrometheusMetric;

const E2_MIN: usize = 8;
const E2_MAX: usize = 36;
const BUCKETS_COUNT: usize = E2_MAX - E2_MIN + 2;

/// 2 ^ i - 1
const BUCKET_UPPER_BOUNDS: [u64; BUCKETS_COUNT] = [
    255,
    511,
    1023,
    2047,
    4095,
    8191,
    16383,
    32767,
    65535,
    131071,
    262143,
    524287,
    1048575,
    2097151,
    4194303,
    8388607,
    16777215,
    33554431,
    67108863,
    134217727,
    268435455,
    536870911,
    1073741823,
    2147483647,
    4294967295,
    8589934591,
    17179869183,
    34359738367,
    68719476735,
    u64::MAX,
];

fn get_vmrange(bucket_no: usize) -> String {
    return if bucket_no == 0 {
        format!("{}...{}", 0, BUCKET_UPPER_BOUNDS.first().unwrap())
    } else if bucket_no == BUCKETS_COUNT - 1 {
        format!(
            "{}...+Inf",
            BUCKET_UPPER_BOUNDS[BUCKET_UPPER_BOUNDS.len() - 2] + 1
        )
    } else {
        format!(
            "{}...{}",
            BUCKET_UPPER_BOUNDS[bucket_no - 1] + 1,
            BUCKET_UPPER_BOUNDS[bucket_no],
        )
    };
}

fn get_bucket_no(value: u64) -> usize {
    for (no, upper) in BUCKET_UPPER_BOUNDS.iter().enumerate() {
        if value <= *upper {
            return no;
        }
    }
    // should never actually happen
    return BUCKETS_COUNT - 1;
}

pub struct Histogram {
    buckets: [u64; BUCKETS_COUNT],
    count: u64,
    sum: u64,
    tags: Vec<Tag>,
    generation: u64,
    name: String,
}

impl Histogram {
    pub fn new(name: String, tags: Vec<Tag>) -> Self {
        return Self {
            buckets: [0u64; BUCKETS_COUNT],
            count: 0,
            sum: 0,
            tags,
            name,
            generation: 1,
        };
    }

    /// reset histogram and bump it's generation
    pub fn reset(&mut self) {
        self.sum = 0;
        self.count = 0;
        self.buckets = [0u64; BUCKETS_COUNT];
        self.generation += 1;
    }

    /// Track value
    /// Histogram resets at integer overflow because counters should better be monotonic
    pub fn track(&mut self, value: u64) {
        let bucket_no = get_bucket_no(value);
        let result_bucket = self.buckets[bucket_no].checked_add(value);
        let result_sum = self.sum.checked_add(value);

        let mut should_reset = true;

        if let Some(result_sum) = result_sum {
            if let Some(result_bucket) = result_bucket {
                should_reset = false;
                self.buckets[bucket_no] = result_bucket;
                self.sum = result_sum;
                self.count += 1;
            }
        }

        if should_reset {
            self.reset();
        }
    }
}

impl PrometheusMetric for Histogram {
    fn serialize_prometheus(&self) -> Vec<String> {
        let mut result: Vec<String> = Vec::with_capacity(self.buckets.len() + 2); // + total + count

        let mut common_tags = String::with_capacity(128); // heuristic
        common_tags.push_str(&*format!("{{generation=\"{}\"", self.generation));
        for tag in &self.tags {
            common_tags.push(',');
            common_tags.push_str(&*format!("{}=\"{}\"", tag.key, tag.value))
        }
        common_tags.shrink_to_fit();

        for (bucket_no, bucket) in self.buckets.iter().enumerate() {
            if *bucket != 0u64 {
                let mut line = String::with_capacity(256);
                line.push_str(&format!("{}_bucket", &self.name));
                line.push_str(&common_tags);
                line.push_str(&format!(
                    ",{}=\"{}\"}} {}",
                    "vmrange",
                    &get_vmrange(bucket_no),
                    *bucket
                ));

                line.shrink_to_fit();
                result.push(line)
            }
        }

        let mut count = String::with_capacity(256);
        count.push_str(&format!("{}_count", &self.name));
        count.push_str(&common_tags);
        count.push_str(&format!("}} {}", self.count));
        count.shrink_to_fit();
        result.push(count);

        let mut sum = String::with_capacity(256);
        sum.push_str(&format!("{}_sum", &self.name));
        sum.push_str(&common_tags);
        sum.push_str(&format!("}} {}", self.sum));
        sum.shrink_to_fit();
        result.push(sum);

        result.shrink_to_fit();
        result
    }
}

#[cfg(test)]
mod tests {
    use crate::lib::metrics::histogram::metric::{get_vmrange, Histogram, BUCKETS_COUNT};
    use crate::lib::metrics::tag::Tag;
    use crate::lib::metrics::traits::PrometheusMetric;

    #[test]
    fn test_vmrange_min() {
        let bucket_no = 0usize;

        let vmrange = get_vmrange(bucket_no);

        assert_eq!(vmrange, "0...255")
    }

    #[test]
    fn test_vmrange_max() {
        let bucket_no = BUCKETS_COUNT - 1;

        let vmrange = get_vmrange(bucket_no);

        assert_eq!(vmrange, "68719476736...+Inf")
    }

    #[test]
    fn test_vmrange_5() {
        let bucket_no = 5usize;

        let vmrange = get_vmrange(bucket_no);

        assert_eq!(vmrange, "4096...8191")
    }

    #[test]
    fn test_vmrange_6() {
        let bucket_no = 6usize;

        let vmrange = get_vmrange(bucket_no);

        assert_eq!(vmrange, "8192...16383")
    }

    #[test]
    fn test_overflow_sum() {
        let mut histogram = Histogram::new(String::from("hist"), Vec::new());
        histogram.track(1);
        histogram.track(2);

        histogram.track(u64::MAX - 1);

        assert_eq!(histogram.generation, 2);
        assert_eq!(histogram.sum, 0);
        assert_eq!(histogram.count, 0);
    }

    #[test]
    fn test_overflow_bucket() {
        let mut histogram = Histogram::new(String::from("hist"), Vec::new());
        histogram.track(u64::MAX - 1);

        histogram.track(u64::MAX - 1);

        assert_eq!(histogram.generation, 2);
        assert_eq!(histogram.sum, 0);
        assert_eq!(histogram.count, 0);
    }

    #[test]
    fn test_track_1() {
        let mut histogram = Histogram::new(String::from("hist"), Vec::new());

        histogram.track(1);

        assert_eq!(histogram.sum, 1);
        assert_eq!(histogram.count, 1);
        assert_eq!(histogram.buckets[0], 1);
    }

    #[test]
    fn test_track_1_256() {
        let mut histogram = Histogram::new(String::from("hist"), Vec::new());

        histogram.track(1);
        histogram.track(256);

        assert_eq!(histogram.sum, 257);
        assert_eq!(histogram.count, 2);
        assert_eq!(histogram.buckets[0], 1);
        assert_eq!(histogram.buckets[1], 256);
    }

    #[test]
    fn test_serialize_prometheus_empty() {
        let histogram = Histogram::new(String::from("hist"), Vec::new());

        let res = histogram.serialize_prometheus();

        assert_eq!(res.len(), 2);
        assert_eq!(res[0], String::from("hist_count{generation=\"1\"} 0"));
        assert_eq!(res[1], String::from("hist_sum{generation=\"1\"} 0"));
    }

    #[test]
    fn test_serialize_prometheus_buckets() {
        let mut histogram = Histogram::new(String::from("hist"), Vec::new());
        histogram.track(1);
        histogram.track(256);
        histogram.track(512);

        let res = histogram.serialize_prometheus();

        assert_eq!(res.len(), 5);
        assert_eq!(
            res[0],
            String::from("hist_bucket{generation=\"1\",vmrange=\"0...255\"} 1")
        );
        assert_eq!(
            res[1],
            String::from("hist_bucket{generation=\"1\",vmrange=\"256...511\"} 256")
        );
        assert_eq!(
            res[2],
            String::from("hist_bucket{generation=\"1\",vmrange=\"512...1023\"} 512")
        );
        assert_eq!(res[3], String::from("hist_count{generation=\"1\"} 3"));
        assert_eq!(res[4], String::from("hist_sum{generation=\"1\"} 769"));
    }

    #[test]
    fn test_serialize_with_tags() {
        let mut histogram = Histogram::new(
            String::from("hist"),
            vec![Tag {
                key: String::from("key"),
                value: String::from("value"),
            }],
        );

        histogram.track(1);

        let res = histogram.serialize_prometheus();

        assert_eq!(res.len(), 3);
        assert_eq!(
            res[0],
            String::from("hist_bucket{generation=\"1\",key=\"value\",vmrange=\"0...255\"} 1")
        );
        assert_eq!(
            res[1],
            String::from("hist_count{generation=\"1\",key=\"value\"} 1")
        );
        assert_eq!(
            res[2],
            String::from("hist_sum{generation=\"1\",key=\"value\"} 1")
        );
    }
}
