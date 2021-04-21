use std::sync::mpsc::{Receiver, Sender};
use palantir_proto::palantir::request::Request;
use palantir_proto::palantir::request::request::Message as ProtoMessage;
use crate::config::defs::RouterConfig;
use palantir_proto::palantir::apm::v1::action::ApmV1Action;
use log::{error};
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

/// Reading incoming requests from potentially many listeners and routes them
/// to appropriate registry handler based on checksum
pub struct Router {
    rx: Receiver<ProtoMessage>,
    tx_list: Vec<Sender<ProtoMessage>>,
    config: RouterConfig,
}

fn generate_checksum(m: ProtoMessage) -> u64 {
    let mut hasher = DefaultHasher::new();
    match m {
        ProtoMessage::ApmV1Action(action) => {
            hasher.write(action.realm.as_bytes());
            hasher.write(action.application.as_bytes());
            hasher.write(action.application_hash.as_bytes());
            hasher.write(action.action_kind.as_bytes());
            hasher.write(action.action_name.as_bytes());
        }
    }
    hasher.finish()
}

impl Router {
    pub fn new(rx: Receiver<ProtoMessage>, tx_list: Vec<Sender<ProtoMessage>>, config: RouterConfig) -> Self {
        Self { rx, tx_list, config }
    }
    
    /// blocks current thread in request routing loop
    pub fn run(&self) {
        match self.rx.recv() {
            Ok(message) => {
                let checksum = generate_checksum(message);
                
            },
            Err(e ) => {
                error!("Message can't be read from channel");
                // we are unable to operate normally with dropped sender
                // and there is also no way to re-init whole data pipeline
                // TODO introduce mechanism of re-creating data pipeline
                std::process::exit(1);
            }
        }
    }
}
