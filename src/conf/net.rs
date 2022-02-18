use serde::{Serialize, Deserialize};
use super::Config;
use crate::utils::{ConnectOpts, HaproxyOpts};
use crate::utils::{TCP_TIMEOUT, UDP_TIMEOUT};
use crate::utils::PROXY_PROTOCOL_VERSION;
use crate::utils::PROXY_PROTOCOL_TIMEOUT;

#[derive(Serialize, Debug, Deserialize, Clone, Copy, Default)]
pub struct NetConf {
    #[serde(default)]
    pub use_udp: Option<bool>,

    #[serde(default)]
    pub fast_open: Option<bool>,

    #[serde(default)]
    pub zero_copy: Option<bool>,

    #[serde(default)]
    pub send_proxy: Option<bool>,

    #[serde(default)]
    pub accept_proxy: Option<bool>,

    #[serde(default)]
    pub send_proxy_version: Option<usize>,

    #[serde(default)]
    pub accept_proxy_timeout: Option<usize>,

    #[serde(default)]
    pub tcp_timeout: Option<usize>,

    #[serde(default)]
    pub udp_timeout: Option<usize>,
}

impl Config for NetConf {
    type Output = ConnectOpts;

    fn build(self) -> Self::Output {
        macro_rules! unbox {
            ($field: ident) => {
                self.$field.unwrap_or_default()
            };
            ($field: ident, $value: expr) => {
                self.$field.unwrap_or($value)
            };
        }

        let use_udp = unbox!(use_udp);

        let fast_open = unbox!(fast_open);
        let zero_copy = unbox!(zero_copy);

        let tcp_timeout = unbox!(tcp_timeout, TCP_TIMEOUT);
        let udp_timeout = unbox!(udp_timeout, UDP_TIMEOUT);

        let send_proxy = unbox!(send_proxy);
        let send_proxy_version =
            unbox!(send_proxy_version, PROXY_PROTOCOL_VERSION);

        let accept_proxy = unbox!(accept_proxy);
        let accept_proxy_timeout =
            unbox!(accept_proxy_timeout, PROXY_PROTOCOL_TIMEOUT);

        ConnectOpts {
            use_udp,
            fast_open,
            zero_copy,
            tcp_timeout,
            udp_timeout,

            // from endpoint
            send_through: None,
            bind_interface: None,

            haproxy_opts: HaproxyOpts {
                send_proxy,
                accept_proxy,
                send_proxy_version,
                accept_proxy_timeout,
            },
        }
    }

    fn rst_field(&mut self, other: &Self) -> &mut Self {
        use crate::rst;
        let other = *other;

        rst!(self, use_udp, other);
        rst!(self, fast_open, other);
        rst!(self, zero_copy, other);
        rst!(self, tcp_timeout, other);
        rst!(self, udp_timeout, other);
        rst!(self, send_proxy, other);
        rst!(self, accept_proxy, other);
        rst!(self, send_proxy_version, other);
        rst!(self, accept_proxy_timeout, other);
        self
    }

    fn take_field(&mut self, other: &Self) -> &mut Self {
        use crate::take;
        let other = *other;

        take!(self, use_udp, other);
        take!(self, fast_open, other);
        take!(self, zero_copy, other);
        take!(self, tcp_timeout, other);
        take!(self, udp_timeout, other);
        take!(self, send_proxy, other);
        take!(self, accept_proxy, other);
        take!(self, send_proxy_version, other);
        take!(self, accept_proxy_timeout, other);
        self
    }

    fn from_cmd_args(matches: &clap::ArgMatches) -> Self {
        macro_rules! unpack {
            ($key: expr) => {
                if matches.is_present($key) {
                    Some(true)
                } else {
                    None
                }
            };
            ($key: expr, $t: ident) => {
                matches.value_of($key).map(|x| x.parse::<$t>().unwrap())
            };
        }

        let use_udp = unpack!("use_udp");
        let fast_open = unpack!("fast_open");
        let zero_copy = unpack!("zero_copy");

        let tcp_timeout = unpack!("tcp_timeout", usize);
        let udp_timeout = unpack!("udp_timeout", usize);

        let send_proxy = unpack!("send_proxy");
        let send_proxy_version = unpack!("send_proxy_version", usize);

        let accept_proxy = unpack!("accept_proxy");
        let accept_proxy_timeout = unpack!("accept_proxy_timeout", usize);

        Self {
            use_udp,
            fast_open,
            zero_copy,
            tcp_timeout,
            udp_timeout,
            send_proxy,
            accept_proxy,
            send_proxy_version,
            accept_proxy_timeout,
        }
    }
}
