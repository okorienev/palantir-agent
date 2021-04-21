use log::{info, LevelFilter};
use palantir_agent_lib::config::defs::{Config, ListenerType, UDPConfig};
use palantir_agent_lib::metrics::histogram::builder::HistogramBuilder;
use palantir_agent_lib::metrics::traits::PrometheusMetric;
use palantir_agent_lib::workers::registry::apm::APMRegistry;
use palantir_agent_lib::workers::server::Server;
use simple_logger::SimpleLogger;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

fn main() {
    SimpleLogger::new()
        .with_level(LevelFilter::Trace)
        .init()
        .unwrap();
    let config = Config {
        listeners: vec![
            ListenerType::UDP(UDPConfig {
                port: 5545,
                buffer_size: 4096,
            }),
            ListenerType::UDP(UDPConfig {
                port: 5546,
                buffer_size: 4096,
            }),
        ],
    };

    let (tx, rx) = channel();

    let server = Server::new(config.listeners, tx);
    let listener_handles = server.schedule().unwrap();

    let mut registry = APMRegistry::new(rx);
    let registry_handler = thread::spawn(move || {
        registry.run();
    });

    for join_handle in listener_handles {
        join_handle.join();
    }
    registry_handler.join();
}
