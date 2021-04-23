use std::sync::{Arc, Mutex, TryLockError, PoisonError};
use std::collections::HashMap;
use crate::util::checksum::Checksum;
use crate::workers::registry::hc::HistogramCollection;
use crate::metrics::histogram::metric::Histogram;
use std::sync::mpsc::{Receiver, TryRecvError, RecvError};
use palantir_proto::palantir::request::request::Message as ProtoMessage;
use std::error::Error;
use std::time::Instant;
use log::{error, trace};
use std::any::Any;

pub enum ProcessError {
    Disconnected,
    LockPoisoned,
}

impl From<PoisonError<()>> for ProcessError {
    fn from(_: PoisonError<()>) -> Self {
        return Self::LockPoisoned
    }
}

impl From<RecvError> for ProcessError {
    fn from(_: RecvError) -> Self {
        return Self::Disconnected
    }
}

pub struct Processor {
    rx: Receiver<ProtoMessage>,
    client_metrics: Arc<Mutex<HashMap<u64, HistogramCollection>>>,
    handle_time: Arc<Mutex<Histogram>>,

    keepalive_reporter: Receiver<()>,
}

impl Processor {
    pub fn new(
        rx: Receiver<ProtoMessage>,
        client_metrics: Arc<Mutex<HashMap<u64, HistogramCollection>>>,
        handle_time: Arc<Mutex<Histogram>>,
        keepalive_reporter: Receiver<()>,
    ) -> Self {
        Self {rx, client_metrics, handle_time, keepalive_reporter}
    }

    pub fn run(&mut self) -> Result<(), ProcessError<>> {
        loop {
            let reporter_alive = self.keepalive_reporter.try_recv();
            if let Err(TryRecvError::Disconnected) = reporter_alive {
                error!("Reporter panicked, exiting");
                return Err(ProcessError::Disconnected)
            }

            self.tick()?;
        }
    }

    fn tick(&mut self) -> Result<(), ProcessError> {
        let msg = self.rx.recv()?;
        trace!("message received by registry");
        let now = Instant::now();
        let checksum = msg.checksum();

        let mut locked = self.client_metrics.lock().unwrap();
        let hc = locked
            .entry(checksum)
            .or_insert(HistogramCollection::from(&msg));
        hc.process(msg);

        let elapsed = now.elapsed();
        trace!("processing took {} us", elapsed.as_micros());
        let mut locked = self.handle_time.lock().unwrap();
        locked.track(elapsed.as_micros() as u64);

        return Ok(());
    }
}
    