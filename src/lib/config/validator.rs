use crate::config::defs::{Config, ListenerType};
use crate::config::parser::LogicError;
use std::collections::HashSet;

/// checks that no port is used twice
fn listeners_no_same_ports(listeners_config: &Vec<ListenerType>) -> Result<(), LogicError> {
    let mut ports: HashSet<u16> = HashSet::new();

    for listener in listeners_config {
        match &listener {
            ListenerType::UDP(cfg) => {
                if ports.contains(&cfg.port) {
                    return Err(LogicError::PortUsedTwice(cfg.port));
                }
                ports.insert(cfg.port);
            }
            ListenerType::TCP(cfg) => {
                if ports.contains(&cfg.port) {
                    return Err(LogicError::PortUsedTwice(cfg.port));
                }
                ports.insert(cfg.port);
            }
        }
    }

    Ok(())
}

/// checks that there is at least one configured listener
fn listeners_at_least_one(listeners_config: &Vec<ListenerType>) -> Result<(), LogicError> {
    if listeners_config.is_empty() {
        return Err(LogicError::AtLeastOneListener);
    }
    Ok(())
}

#[allow(dead_code)]
pub fn run_validation_chain(config: &Config) -> Result<(), LogicError> {
    listeners_at_least_one(&config.listeners)?;
    listeners_no_same_ports(&config.listeners)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::config::defs::{Config, ListenerType, ReporterConfig, TCPConfig, UDPConfig};
    use crate::config::parser::LogicError;
    use crate::config::validator::run_validation_chain;

    #[test]
    fn test_no_listeners_invalid() {
        let config = Config {
            listeners: vec![],
            reporter: ReporterConfig {
                vm_import_url: "".to_string(),
            },
        };

        let result = run_validation_chain(&config).err().unwrap();

        match result {
            LogicError::PortUsedTwice(_) => {
                panic!("wrong error")
            }
            LogicError::AtLeastOneListener => (),
        }
    }

    #[test]
    fn test_port_used_twice() {
        let config = Config {
            listeners: vec![
                ListenerType::UDP(UDPConfig {
                    port: 2746,
                    buffer_size: 4096,
                }),
                ListenerType::TCP(TCPConfig {
                    port: 2746,
                    buffer_size: 4096,
                }),
            ],
            reporter: ReporterConfig {
                vm_import_url: "".to_string(),
            },
        };

        let result = run_validation_chain(&config).err().unwrap();

        match result {
            LogicError::PortUsedTwice(port) => {
                assert_eq!(port, 2746)
            }
            LogicError::AtLeastOneListener => {
                panic!("wrong error")
            }
        }
    }

    #[test]
    fn test_ok() {
        let config = Config {
            listeners: vec![
                ListenerType::UDP(UDPConfig {
                    port: 2746,
                    buffer_size: 4096,
                }),
                ListenerType::TCP(TCPConfig {
                    port: 2747,
                    buffer_size: 4096,
                }),
            ],
            reporter: ReporterConfig {
                vm_import_url: "".to_string(),
            },
        };

        let _result = run_validation_chain(&config).ok().unwrap();
    }
}
