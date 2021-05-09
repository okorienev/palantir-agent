use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub listeners: Vec<ListenerType>,
    pub reporter: ReporterConfig,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ReporterConfig {
    pub vm_import_url: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum ListenerType {
    UDP(UDPConfig),
    TCP(TCPConfig),
}

fn default_buffer_size() -> u16 {
    4096
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct UDPConfig {
    pub port: u16,
    #[serde(default = "default_buffer_size")]
    pub buffer_size: u16,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TCPConfig {
    pub port: u16,
    #[serde(default = "default_buffer_size")]
    pub buffer_size: u16,
}
