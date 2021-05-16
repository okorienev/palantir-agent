use crate::metrics::histogram::metric::Histogram;
use crate::metrics::traits::PrometheusMetric;
use crate::workers::registry::error::RegistryError;
use crate::workers::registry::hc::HistogramCollection;
use hyper::{Body, Client, Request};
use log::{error, info, trace};
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const REPORT_PERIOD_SECONDS: u64 = 10;

// TODO make configurable
// TODO add metrics about report generation time
// TODO add metrics about victoriametrics response time
// TODO add reading shared labels from
pub struct Reporter<'a> {
    client_metrics: Arc<Mutex<HashMap<u64, HistogramCollection>>>,
    handle_time: Arc<Mutex<Histogram>>,

    keepalive_tx: Sender<()>,

    vm_import_url: &'a str,
}

impl Reporter<'_> {
    pub fn new(
        client_metrics: Arc<Mutex<HashMap<u64, HistogramCollection>>>,
        handle_time: Arc<Mutex<Histogram>>,
        keepalive_tx: Sender<()>,
        vm_import_url: &'static str,
    ) -> Self {
        Self {
            client_metrics,
            handle_time,
            keepalive_tx,
            vm_import_url,
        }
    }

    pub async fn run(&mut self) -> Result<(), RegistryError> {
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

            self.tick().await?;
            info!("Report took {}ms", start.elapsed().as_millis());
            thread::sleep(Duration::from_secs(REPORT_PERIOD_SECONDS));
        }
    }

    async fn tick(&mut self) -> Result<(), RegistryError> {
        // todo this is very very bad (tons of allocations)
        // maybe write all the data to the tempfile and then use it as request body?
        // or integrate hyper::body::Body::channel normally?
        let mut report = String::new();
        let locked = self.handle_time.lock().unwrap();
        for row in locked.serialize_prometheus() {
            report.push_str(&row);
        }
        std::mem::drop(locked);

        let locked = self.client_metrics.lock().unwrap();
        for hc in locked.values() {
            for row in hc.serialize_prometheus() {
                report.push_str(&row);
            }
        }
        std::mem::drop(locked);

        let client = Client::new();
        let req = Request::post(self.vm_import_url).body(Body::from(report));

        match req {
            Ok(request) => {
                let response = client.request(request).await;
                match response {
                    Ok(result) => {
                        info!("got {} from vm", result.status().as_u16())
                    }
                    Err(err) => {
                        error!("Error during request {:?}", err);
                    }
                }
            }
            Err(err) => {
                error!("Error during building request {:?}", err);
            }
        };

        Ok(())
    }
}
