use crate::metrics::histogram::metric::Histogram;
use crate::workers::registry::hc::HistogramCollection;
use log::{error, trace};
use palantir_proto::palantir::request::request::Message as ProtoMessage;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, channel};
use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex};
use std::thread;
use crate::workers::registry::processor::Processor;

pub fn run_registry(rx: Receiver<ProtoMessage>) -> thread::Result<()> {
    let client_metrics: Arc<Mutex<HashMap<u64, HistogramCollection>>> = Arc::new(Mutex::new(HashMap::new()));
    let handle_time: Arc<Mutex<Histogram>> = Arc::new(Mutex::new(Histogram::new("request_handle_time".to_string(), Vec::new())));

    let (reporter_tx, reporter_rx) = channel();
    let processor_handle =  thread::spawn(move || {
        let mut processor = Processor::new(
            rx,
            client_metrics.clone(),
            handle_time.clone(),
                reporter_rx,
            );
            processor.run();
        });

        let reporter_handle = thread::spawn(move || {
            let tx = reporter_tx;
            thread::sleep(Duration::from_secs(10));
        });

        processor_handle.join()?;
        reporter_handle.join()?;

        Ok(())
}
