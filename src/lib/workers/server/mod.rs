use crate::config::defs::ListenerType;
use listeners::udp::UDPListener;
use palantir_proto::palantir::request::request::Message as ProtoMessage;
use std::io::Result as IOResult;
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::JoinHandle;

mod listeners;

pub struct Server {
    listeners: Vec<ListenerType>,
    tx: Sender<ProtoMessage>,
}

impl Server {
    pub fn new(listeners: Vec<ListenerType>, tx: Sender<ProtoMessage>) -> Self {
        return Self { listeners, tx };
    }

    pub fn schedule(&self) -> IOResult<Vec<JoinHandle<()>>> {
        let mut threads = Vec::new();
        for config in &self.listeners {
            match config {
                ListenerType::UDP(udp_config) => {
                    let listener = UDPListener::new(udp_config, self.tx.clone())?;
                    threads.push(thread::spawn(move || {
                        listener.run();
                    }));
                }
                ListenerType::TCP(_) => {
                    panic!("not implemented")
                }
            }
        }

        return Ok(threads);
    }
}
