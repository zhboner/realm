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

    #[serde(default = "df_timeout")]
    timeout: usize,

    local: String,

    remote: String,

    #[serde(default)]
    through: String,
}

const fn df_timeout() -> usize {
    crate::utils::TIMEOUT
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
            self.timeout,
        )
    }
}
