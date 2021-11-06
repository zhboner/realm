use std::fs;

use serde::{Serialize, Deserialize};

mod dns;
mod endpoint;
pub use dns::DnsMode;
pub use endpoint::EndpointConfig;

#[derive(Debug, Serialize, Deserialize)]
pub struct FullConfig {
    #[serde(default)]
    pub dns_mode: DnsMode,
    pub endpoints: Vec<EndpointConfig>,
}

impl FullConfig {
    pub fn from_config_file(file: &str) -> Self {
        let config = fs::read_to_string(file)
            .unwrap_or_else(|_| panic!("unable to open {}", file));
        serde_json::from_str(&config)
            .unwrap_or_else(|_| panic!("unable to open {}", file))
    }
}
