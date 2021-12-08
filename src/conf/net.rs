use serde::{Serialize, Deserialize};

#[derive(Serialize, Debug, Deserialize, Clone, Copy, Default)]
pub struct NetConf {
    #[serde(default)]
    pub udp: Option<bool>,

    #[serde(default)]
    pub fast_open: Option<bool>,

    #[serde(default)]
    pub zero_copy: Option<bool>,

    #[serde(default)]
    pub tcp_timeout: Option<bool>,

    #[serde(default)]
    pub udp_timeout: Option<bool>,
}
