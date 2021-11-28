use serde::{Serialize, Deserialize};
use crate::utils::Endpoint;

#[derive(Debug, Serialize, Deserialize)]
pub struct EndpointConf {
    #[serde(default)]
    pub udp: bool,

    #[serde(default)]
    pub fast_open: bool,

    #[serde(default)]
    pub zero_copy: bool,

    #[serde(default = "tcp_timeout")]
    pub tcp_timeout: usize,

    #[serde(default = "udp_timeout")]
    pub udp_timeout: usize,

    pub local: String,

    pub remote: String,

    #[serde(default)]
    pub through: String,
}

const fn tcp_timeout() -> usize {
    crate::utils::TCP_TIMEOUT
}

const fn udp_timeout() -> usize {
    crate::utils::UDP_TIMEOUT
}

impl EndpointConf {
    pub fn build(&self) -> Endpoint {
        Endpoint::new(
            &self.local,
            &self.remote,
            &self.through,
            self.udp,
            self.fast_open,
            self.zero_copy,
            self.tcp_timeout,
            self.udp_timeout,
        )
    }
}
