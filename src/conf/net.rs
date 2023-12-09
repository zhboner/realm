use serde::{Serialize, Deserialize};
use realm_core::endpoint::{ConnectOpts, ProxyOpts};

use super::Config;
use crate::consts::{TCP_KEEPALIVE, TCP_TIMEOUT, UDP_TIMEOUT};
use crate::consts::PROXY_PROTOCOL_VERSION;
use crate::consts::PROXY_PROTOCOL_TIMEOUT;

#[derive(Serialize, Debug, Deserialize, Clone, Copy, Default)]
pub struct NetConf {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_tcp: Option<bool>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_udp: Option<bool>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_proxy: Option<bool>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accept_proxy: Option<bool>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_proxy_version: Option<usize>,

    #[serde(default)]
    pub accept_proxy_timeout: Option<usize>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcp_keepalive: Option<u64>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcp_timeout: Option<usize>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub udp_timeout: Option<usize>,
}

#[derive(Debug)]
pub struct NetInfo {
    pub conn_opts: ConnectOpts,
    pub no_tcp: bool,
    pub use_udp: bool,
}

impl Config for NetConf {
    type Output = NetInfo;

    fn is_empty(&self) -> bool {
        crate::empty![self =>
            send_proxy, accept_proxy, send_proxy_version, accept_proxy_timeout,
            tcp_keepalive, tcp_timeout, udp_timeout
        ]
    }

    fn build(self) -> Self::Output {
        macro_rules! unbox {
            ($field: ident) => {
                self.$field.unwrap_or_default()
            };
            ($field: ident, $value: expr) => {
                self.$field.unwrap_or($value)
            };
        }

        let no_tcp = unbox!(no_tcp);
        let use_udp = unbox!(use_udp);
        let tcp_keepalive = unbox!(tcp_keepalive, TCP_KEEPALIVE);
        let tcp_timeout = unbox!(tcp_timeout, TCP_TIMEOUT);
        let udp_timeout = unbox!(udp_timeout, UDP_TIMEOUT);

        let send_proxy = unbox!(send_proxy);
        let send_proxy_version = unbox!(send_proxy_version, PROXY_PROTOCOL_VERSION);

        let accept_proxy = unbox!(accept_proxy);
        let accept_proxy_timeout = unbox!(accept_proxy_timeout, PROXY_PROTOCOL_TIMEOUT);

        let conn_opts = ConnectOpts {
            tcp_keepalive: tcp_keepalive,
            connect_timeout: tcp_timeout,
            associate_timeout: udp_timeout,

            // from endpoint
            bind_address: None,
            bind_interface: None,

            #[cfg(feature = "balance")]
            balancer: Default::default(),

            #[cfg(feature = "transport")]
            transport: None,

            proxy_opts: ProxyOpts {
                send_proxy,
                accept_proxy,
                send_proxy_version,
                accept_proxy_timeout,
            },
        };

        NetInfo {
            conn_opts,
            no_tcp,
            use_udp,
        }
    }

    fn rst_field(&mut self, other: &Self) -> &mut Self {
        use crate::rst;
        let other = *other;

        rst!(self, no_tcp, other);
        rst!(self, use_udp, other);
        rst!(self, tcp_keepalive, other);
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

        take!(self, no_tcp, other);
        take!(self, use_udp, other);
        take!(self, tcp_keepalive, other);
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

        let no_tcp = unpack!("no_tcp");
        let use_udp = unpack!("use_udp");

        let tcp_keepalive = unpack!("tcp_keepalive", u64);
        let tcp_timeout = unpack!("tcp_timeout", usize);
        let udp_timeout = unpack!("udp_timeout", usize);

        let send_proxy = unpack!("send_proxy");
        let send_proxy_version = unpack!("send_proxy_version", usize);

        let accept_proxy = unpack!("accept_proxy");
        let accept_proxy_timeout = unpack!("accept_proxy_timeout", usize);

        Self {
            no_tcp,
            use_udp,
            tcp_keepalive,
            tcp_timeout,
            udp_timeout,
            send_proxy,
            accept_proxy,
            send_proxy_version,
            accept_proxy_timeout,
        }
    }
}
