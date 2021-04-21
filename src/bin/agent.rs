use log::{info, LevelFilter};
use palantir_agent_lib::config::defs::{Config, ListenerType, UDPConfig, RouterConfig};
use palantir_agent_lib::metrics::histogram::builder::HistogramBuilder;
use palantir_agent_lib::metrics::traits::PrometheusMetric;
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
        router: RouterConfig {
            inactivity_delay_ms: 10,
        }
    };

    let (tx, rx) = channel();

    let server = Server::new(config.listeners, tx);
    let listener_handles = server.schedule().unwrap();

    let receiver_handle = thread::spawn(move || loop {
        let msg = rx.recv().unwrap();
        info!("{:?}", msg);
    });

    receiver_handle.join();
    for join_handle in listener_handles {
        join_handle.join();
    }
}
