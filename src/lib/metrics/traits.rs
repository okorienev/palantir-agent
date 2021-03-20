pub trait PrometheusMetric {
    fn serialize_prometheus(&self) -> Vec<String>;
}
