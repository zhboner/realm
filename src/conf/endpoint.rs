use serde::{Serialize, Deserialize};
use crate::utils::Endpoint;

#[derive(Debug, Serialize, Deserialize)]
pub struct EndpointConfig {
    #[serde(default)]
    udp: bool,
    local: String,
    remote: String,
    #[serde(default)]
    through: String,
}

impl EndpointConfig {
    pub fn build(&self) -> Endpoint {
        Endpoint::new(&self.local, &self.remote, &self.through, self.udp)
    }
}
