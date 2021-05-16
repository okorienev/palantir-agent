use crate::constants as c;
use crate::metrics::histogram::metric::Histogram;
use crate::metrics::traits::PrometheusMetric;
use crate::workers::registry::error::RegistryError;
use crate::workers::registry::hc::HistogramCollection;
use hyper::{Body, Client, Request};
use lazy_static::lazy_static;
use log::{error, info, trace, warn};
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use url::Url;

const REPORT_PERIOD_SECONDS: u64 = 10;

lazy_static! {
    static ref ADDITIONAL_LABELS: Vec<(String, String)> = {
        let mut labels = Vec::new();
        for (key, value) in std::env::vars() {
            if !key.starts_with(c::EXTRA_LABEL_PREFIX) {
                trace!("key {} omitted", key);
                continue;
            }
            if !c::EXTRA_LABEL_REGEX.is_match(&key) || !c::EXTRA_LABEL_REGEX.is_match(&value) {
                warn!("pair {}:{} is invalid", key, value);
                continue;
            }
            let label_name = key.replace(c::EXTRA_LABEL_PREFIX, "").to_lowercase();
            trace!("adding extra label {}:{}", label_name, value);
            labels.push((label_name, value));
        }
        labels
    };
}

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
        // this is not recoverable - no sense in working if there is nowhere to report
        // also config validation should check that url is OK
        let mut url = Url::parse(self.vm_import_url).unwrap();
        for (key, value) in ADDITIONAL_LABELS.iter() {
            url.query_pairs_mut().append_pair(key, value);
        }
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
