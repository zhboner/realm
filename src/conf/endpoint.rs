use serde::{Serialize, Deserialize};
use crate::utils::Endpoint;

#[derive(Debug, Serialize, Deserialize)]
pub struct EndpointConf {
    #[serde(default)]
    udp: bool,

    #[serde(default)]
    fast_open: bool,

    #[serde(default)]
    zero_copy: bool,

    #[serde(default = "tcp_timeout")]
    tcp_timeout: usize,

    #[serde(default = "udp_timeout")]
    udp_timeout: usize,

    local: String,

    remote: String,

    #[serde(default)]
    through: String,
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
