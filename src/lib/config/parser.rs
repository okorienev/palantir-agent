use super::defs::{Config, ListenerType};
use super::validator::run_validation_chain;
use serde_yaml;
use serde_yaml::Error;
use std::collections::HashSet;
use std::convert::From;

#[derive(Debug)]
pub enum ConfigurationError {
    Parse(serde_yaml::Error),
    Logic(LogicError),
}

impl From<LogicError> for ConfigurationError {
    fn from(err: LogicError) -> Self {
        Self::Logic(err)
    }
}

impl From<serde_yaml::Error> for ConfigurationError {
    fn from(err: Error) -> Self {
        Self::Parse(err)
    }
}

#[derive(Debug)]
pub enum LogicError {
    PortUsedTwice(u16),
    AtLeastOneListener,
}

fn parse_config<T: AsRef<str>>(raw_config: T) -> Result<Config, ConfigurationError> {
    let config: Config = serde_yaml::from_str(raw_config.as_ref())?;
    run_validation_chain(&config)?;

    return Ok(config);
}

#[cfg(test)]
mod tests {
    use crate::config::defs::{Config, ListenerType, TCPConfig, UDPConfig, RouterConfig};
    use crate::config::parser::{parse_config, ConfigurationError, LogicError};
    use serde_yaml::Error;

    #[test]
    fn test_parse_invalid_yaml() {
        let yaml = "@not_a_yaml";

        let result = parse_config(yaml).err().unwrap();

        match result {
            ConfigurationError::Parse(err) => (),
            ConfigurationError::Logic(_) => {
                panic!("Wrong error")
            }
        }
    }

    #[test]
    fn test_parse_correct() {
        let expected_config = Config {
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
            router: RouterConfig {
                inactivity_delay_ms: 10,
            }
        };
        let yaml = "
---
listeners:
  - UDP:
      port: 2746
      buffer_size: 4096
  - TCP:
      port: 2747
router:
  inactivity_delay_ms: 10
        ";
        let result = parse_config(yaml).ok().unwrap();

        assert_eq!(result, expected_config)
    }
}
