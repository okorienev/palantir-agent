use crate::constants as c;
use crate::metrics::histogram::metric::Histogram;
use crate::metrics::tag::Tag;
use crate::util::checksum::Checksum;
use log::warn;
use palantir_proto::palantir::apm::v1::action::ApmV1Action;
use palantir_proto::palantir::request::request::Message as ProtoMessage;
use palantir_proto::palantir::shared::measurement::Measurement as ProtoMeasurement;
use std::collections::HashMap;
use std::time::Instant;

pub struct HistogramCollection {
    tags: Vec<Tag>,
    metrics: HashMap<u64, Histogram>,
    last_hit: Instant,
}

impl HistogramCollection {
    pub fn new(tags: Vec<Tag>) -> Self {
        Self {
            tags,
            metrics: HashMap::new(),
            last_hit: Instant::now(),
        }
    }

    fn process_measurement(&mut self, name: String, took: u64) {
        let checksum = name.checksum();
        match self.metrics.get_mut(&checksum) {
            None => {
                let mut tags = self.tags.clone();
                tags.push(Tag {
                    key: c::ACTION_SPAN_TAG_NAME.to_string(),
                    value: name,
                });
                let mut histogram = Histogram::new(c::ACTION_METRIC_NAME.to_string(), tags);
                histogram.track(took);
                self.metrics.insert(checksum, histogram);
            }
            Some(histogram) => {
                histogram.track(took);
            }
        }
    }

    fn process_measurements(&mut self, measurements: Vec<ProtoMeasurement>, duration: u64) {
        let mut total: u64 = 0;
        for m in measurements {
            total += m.took_us;
            self.process_measurement(m.name, m.took_us);
        }

        match duration.checked_sub(total) {
            None => {
                warn!(
                    "sum of all measurements ({}us) > duration ({}us)",
                    total, duration
                );
                self.process_measurement(c::UNTRACKED_ACTION_KIND_NAME.to_string(), 0);
                self.process_measurement(c::TOTAL_ACTION_KIND_NAME.to_string(), total);
            }
            Some(took) => {
                self.process_measurement(c::UNTRACKED_ACTION_KIND_NAME.to_string(), took);
                self.process_measurement(c::TOTAL_ACTION_KIND_NAME.to_string(), duration);
            }
        }
    }

    pub fn process(&mut self, msg: ProtoMessage) {
        self.last_hit = Instant::now();
        match msg {
            ProtoMessage::ApmV1Action(action) => {
                self.process_measurements(action.measurements, action.total_us);
            }
        }
    }
}

impl From<&ProtoMessage> for HistogramCollection {
    fn from(msg: &ProtoMessage) -> Self {
        match msg {
            ProtoMessage::ApmV1Action(action) => HistogramCollection::from(action),
        }
    }
}

impl From<&ApmV1Action> for HistogramCollection {
    fn from(a: &ApmV1Action) -> Self {
        // TODO drop .clone() usage in some way
        let mut tags = Vec::new();
        tags.push(Tag {
            key: c::REALM_TAG_NAME.to_string(),
            value: a.realm.clone(),
        });
        tags.push(Tag {
            key: c::APPLICATION_TAG_NAME.to_string(),
            value: a.application.clone(),
        });
        tags.push(Tag {
            key: c::APPLICATION_HASH_TAG_NAME.to_string(),
            value: a.application_hash.clone(),
        });
        tags.push(Tag {
            key: c::ACTION_KIND_TAG_NAME.to_string(),
            value: a.action_kind.clone(),
        });
        tags.push(Tag {
            key: c::ACTION_NAME_TAG_NAME.to_string(),
            value: a.action_name.clone(),
        });

        for dimension in &a.additional_dimensions {
            tags.push(Tag::from(dimension));
        }

        HistogramCollection::new(tags)
    }
}
