use crate::constants as c;
use crate::metrics::histogram::builder::HistogramBuilder;
use crate::metrics::histogram::metric::Histogram;
use crate::metrics::tag::Tag;
use crate::util::checksum::Checksum;
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

    fn process_measurements(&mut self, measurements: Vec<ProtoMeasurement>) {
        for m in measurements {
            let checksum = m.name.checksum();
            match self.metrics.get_mut(&checksum) {
                None => {
                    let mut tags = self.tags.clone();
                    tags.push(Tag {
                        key: c::ACTION_SPAN_TAG_NAME.to_string(),
                        value: m.name,
                    });
                    let mut histogram = Histogram::new(c::ACTION_METRIC_NAME.to_string(), tags);
                    histogram.track_n_hits(m.total_us, m.hits as usize);

                    self.metrics.insert(checksum, histogram);
                }
                Some(histogram) => {
                    histogram.track_n_hits(m.total_us, m.hits as usize);
                }
            }
        }
    }

    pub fn process(&mut self, msg: ProtoMessage) {
        self.last_hit = Instant::now();
        match msg {
            ProtoMessage::ApmV1Action(action) => self.process_measurements(action.measurements),
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
        /// TODO drop .clone() usage in some way
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
