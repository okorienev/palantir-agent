use crate::config::defs::ReporterConfig;
use crate::metrics::histogram::metric::Histogram;
use crate::workers::registry::hc::HistogramCollection;
use crate::workers::registry::processor::Processor;
use crate::workers::registry::reporter::Reporter;
use palantir_proto::palantir::request::request::Message as ProtoMessage;
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;

pub fn run_registry(
    rx: Receiver<ProtoMessage>,
    reporter: &'static ReporterConfig,
) -> thread::Result<()> {
    let client_metrics: Arc<Mutex<HashMap<u64, HistogramCollection>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let handle_time: Arc<Mutex<Histogram>> = Arc::new(Mutex::new(Histogram::new(
        "request_handle_time".to_string(),
        Vec::new(),
    )));

    let (reporter_tx, reporter_rx) = channel();

    let metrics_clone = client_metrics.clone();
    let handle_clone = handle_time.clone();
    let processor_handle = thread::spawn(move || {
        let mut processor = Processor::new(rx, metrics_clone, handle_clone, reporter_rx);
        #[allow(unused_must_use)]
        {
            processor.run();
        }
    });

    let metrics_clone = client_metrics.clone();
    let handle_clone = handle_time.clone();
    let reporter_handle = thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Unable to create runtime");

        let mut reporter = Reporter::new(
            metrics_clone,
            handle_clone,
            reporter_tx,
            &reporter.vm_import_url,
        );
        #[allow(unused_must_use)]
        {
            runtime.block_on(reporter.run());
        }
    });

    processor_handle.join()?;
    reporter_handle.join()?;

    Ok(())
}
