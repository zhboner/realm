use serde::{Serialize, Deserialize};
use crate::relay::Endpoint;

#[derive(Debug, Serialize, Deserialize)]
pub struct EndpointConfig {
    udp: bool,
    local: String,
    remote: String,
}

impl EndpointConfig {
    pub fn to_endpoint(self) -> Endpoint {
        Endpoint::new(&self.local, &self.remote, self.udp)
    }
}
