use std::fs;

use serde::{Serialize, Deserialize};

mod log;
pub use self::log::LogConf;

mod dns;
pub use dns::CompatibeDnsConf;

mod endpoint;
pub use endpoint::EndpointConf;

#[derive(Debug, Serialize, Deserialize)]
pub struct FullConf {
    #[serde(default)]
    pub log: LogConf,

    #[serde(default, rename = "dns_mode")]
    pub dns: CompatibeDnsConf,

    pub endpoints: Vec<EndpointConf>,
}

impl FullConf {
    pub fn from_config_file(file: &str) -> Self {
        let config = fs::read_to_string(file)
            .unwrap_or_else(|e| panic!("unable to open {}: {}", file, &e));
        serde_json::from_str(&config)
            .unwrap_or_else(|e| panic!("unable to parse {}: {}", file, &e))
    }
}
