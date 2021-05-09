use crate::metrics::histogram::metric::Histogram;
use crate::metrics::traits::PrometheusMetric;
use crate::workers::registry::error::RegistryError;
use crate::workers::registry::hc::HistogramCollection;
use hyper::http::Error;
use hyper::rt::Future;
use hyper::{Body, Chunk, Client, Request, Response};
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

        let locked = self.handle_time.lock().unwrap();
        for row in locked.serialize_prometheus() {
            writer.send_data(Chunk::from(row));
            writer.send_data(Chunk::from("\n"));
        }
        std::mem::drop(locked);

        let locked = self.client_metrics.lock().unwrap();
        for hc in locked.values() {
            for row in hc.serialize_prometheus() {
                writer.send_data(Chunk::from(row));
                writer.send_data(Chunk::from("\n"));
            }
        }
        std::mem::drop(locked);

        let client = Client::new();
        let req = Request::post(self.vm_import_url).body(body);

        match req {
            Ok(request) => {
                let response = client.request(request).wait();
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
