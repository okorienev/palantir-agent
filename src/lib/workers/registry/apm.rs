use crate::metrics::histogram::metric::Histogram;
use crate::metrics::tag::Tag;
use crate::util::checksum::Checksum;
use crate::workers::registry::hc::HistogramCollection;
use log::{error, trace};
use palantir_proto::palantir::apm::v1::action::ApmV1Action;
use palantir_proto::palantir::request::request::Message as ProtoMessage;
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::time::Instant;

pub struct APMRegistry {
    rx: Receiver<ProtoMessage>,
    client_metrics: HashMap<u64, HistogramCollection>,
    handle_time: Histogram,
}

impl APMRegistry {
    pub fn new(rx: Receiver<ProtoMessage>) -> Self {
        Self {
            rx,
            client_metrics: HashMap::new(),
            handle_time: Histogram::new("request_handle_time".to_string(), Vec::new()),
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.rx.recv() {
                Ok(msg) => {
                    trace!("message received by registry");
                    let now = Instant::now();
                    let checksum = msg.checksum();
                    let mut hc = self
                        .client_metrics
                        .entry(checksum)
                        .or_insert(HistogramCollection::from(&msg));
                    hc.process(msg);
                    let elapsed = now.elapsed();
                    trace!("processing took {} us", elapsed.as_micros());
                    self.handle_time.track(elapsed.as_micros() as u64);
                }
                Err(_) => {
                    error!("Message can't be read from channel");
                    // we are unable to operate normally with dropped receiver
                    // and there is also no way to re-init whole data pipeline
                    // TODO introduce mechanism of re-creating data pipeline
                    std::process::exit(1);
                }
            }
        }
    }
}
