use std::fs;

use serde::{Serialize, Deserialize};

mod dns_mode;
mod endpoint_config;

pub use dns_mode::DnsMode;
pub use endpoint_config::EndpointConfig;

#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalConfig {
    #[serde(default)]
    pub dns_mode: DnsMode,
    pub endpoints: Vec<EndpointConfig>,
}

impl GlobalConfig {
    pub fn from_config_file(file: &str) -> Self {
        let config = fs::read_to_string(file).expect("invalid file path");
        serde_json::from_str(&config).expect("failed to parse config file")
    }
}
