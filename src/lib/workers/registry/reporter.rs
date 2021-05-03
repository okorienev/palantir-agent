use crate::metrics::histogram::metric::Histogram;
use crate::metrics::traits::PrometheusMetric;
use crate::workers::registry::error::RegistryError;
use crate::workers::registry::hc::HistogramCollection;
use hyper::{Body, Chunk};
use log::{error, info, trace};
use std::collections::HashMap;
use std::sync::mpsc::{SendError, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const REPORT_PERIOD_SECONDS: u64 = 10;

// TODO make configurable
// TODO add metrics about report generation time
// TODO add metrics about victoriametrics response time
// TODO add reading shared labels from
pub struct Reporter {
    client_metrics: Arc<Mutex<HashMap<u64, HistogramCollection>>>,
    handle_time: Arc<Mutex<Histogram>>,

    keepalive_tx: Sender<()>,
}

impl Reporter {
    pub fn new(
        client_metrics: Arc<Mutex<HashMap<u64, HistogramCollection>>>,
        handle_time: Arc<Mutex<Histogram>>,
        keepalive_tx: Sender<()>,
    ) -> Self {
        Self {
            client_metrics,
            handle_time,
            keepalive_tx,
        }
    }

    pub fn run(&mut self) -> Result<(), RegistryError> {
        loop {
            trace!("Starting report");
            let start = Instant::now();

            match self.keepalive_tx.send(()) {
                Err(err) => {
                    error!("Registry disconnected");
                    return Err(RegistryError::from(err));
                }
                Ok(_) => {}
            }

            self.tick()?;
            info!("Report took {}ms", start.elapsed().as_millis());
            thread::sleep(Duration::from_secs(REPORT_PERIOD_SECONDS));
        }
    }

    pub fn tick(&mut self) -> Result<(), RegistryError> {
        let (mut writer, body) = Body::channel();

        let mut locked = self.handle_time.lock().unwrap();
        for row in locked.serialize_prometheus() {
            writer.send_data(Chunk::from(row));
        }
        std::mem::drop(locked);

        let mut locked = self.client_metrics.lock().unwrap();
        for hc in locked.values() {
            for row in hc.serialize_prometheus() {
                writer.send_data(Chunk::from(row));
            }
        }
        std::mem::drop(locked);

        Ok(())
    }
}
