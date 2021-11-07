use std::fs;

use serde::{Serialize, Deserialize};

mod endpoint;
pub use endpoint::EndpointConf;

#[cfg(feature = "trust-dns")]
mod dns;
#[cfg(feature = "trust-dns")]
pub use dns::CompatibeDnsConf;

#[derive(Debug, Serialize, Deserialize)]
pub struct FullConf {
    #[cfg(feature = "trust-dns")]
    #[serde(default, rename = "dns_mode")]
    pub dns: CompatibeDnsConf,

    pub endpoints: Vec<EndpointConf>,
}

impl FullConf {
    pub fn from_config_file(file: &str) -> Self {
        let config = fs::read_to_string(file)
            .unwrap_or_else(|_| panic!("unable to open {}", file));
        serde_json::from_str(&config)
            .unwrap_or_else(|_| panic!("unable to parse {}", file))
    }
}
