use serde::{Serialize, Deserialize};
use realm_core::endpoint::{BindOpts, ConnectOpts};

use super::Config;
use crate::consts::{TCP_TIMEOUT, UDP_TIMEOUT};
use crate::consts::{TCP_KEEPALIVE, TCP_KEEPALIVE_PROBE};
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
    pub ipv6_only: Option<bool>,

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
    pub tcp_keepalive: Option<usize>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcp_keepalive_probe: Option<usize>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcp_timeout: Option<usize>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub udp_timeout: Option<usize>,
}

#[derive(Debug)]
pub struct NetInfo {
    pub bind_opts: BindOpts,
    pub conn_opts: ConnectOpts,
    pub no_tcp: bool,
    pub use_udp: bool,
}

impl Config for NetConf {
    type Output = NetInfo;

    fn is_empty(&self) -> bool {
        crate::empty![self =>
            no_tcp, use_udp, ipv6_only,
            send_proxy, accept_proxy, send_proxy_version, accept_proxy_timeout,
            tcp_keepalive, tcp_keepalive_probe, tcp_timeout, udp_timeout
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
        let ipv6_only = unbox!(ipv6_only);
        let tcp_kpa = unbox!(tcp_keepalive, TCP_KEEPALIVE);
        let tcp_kpa_probe = unbox!(tcp_keepalive_probe, TCP_KEEPALIVE_PROBE);
        let tcp_timeout = unbox!(tcp_timeout, TCP_TIMEOUT);
        let udp_timeout = unbox!(udp_timeout, UDP_TIMEOUT);

        let bind_opts = BindOpts {
            ipv6_only,
            bind_interface: None,
        };
        let conn_opts = ConnectOpts {
            tcp_keepalive: tcp_kpa,
            tcp_keepalive_probe: tcp_kpa_probe,
            connect_timeout: tcp_timeout,
            associate_timeout: udp_timeout,

            // from endpoint
            bind_address: None,
            bind_interface: None,

            #[cfg(feature = "balance")]
            balancer: Default::default(),

            #[cfg(feature = "transport")]
            transport: None,

            #[cfg(feature = "proxy")]
            proxy_opts: {
                use realm_core::endpoint::ProxyOpts;
                let send_proxy = unbox!(send_proxy);
                let send_proxy_version = unbox!(send_proxy_version, PROXY_PROTOCOL_VERSION);
                let accept_proxy = unbox!(accept_proxy);
                let accept_proxy_timeout = unbox!(accept_proxy_timeout, PROXY_PROTOCOL_TIMEOUT);
                ProxyOpts {
                    send_proxy,
                    accept_proxy,
                    send_proxy_version,
                    accept_proxy_timeout,
                }
            },
        };

        NetInfo {
            bind_opts,
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
        rst!(self, ipv6_only, other);
        rst!(self, tcp_keepalive, other);
        rst!(self, tcp_keepalive_probe, other);
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
        take!(self, ipv6_only, other);
        take!(self, tcp_keepalive, other);
        take!(self, tcp_keepalive_probe, other);
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
                if matches.get_flag($key) {
                    Some(true)
                } else {
                    None
                }
            };
            ($key: expr, $t: ident) => {
                matches.get_one::<String>($key).and_then(|x| x.parse::<$t>().ok())
            };
        }

        let no_tcp = unpack!("no_tcp");
        let use_udp = unpack!("use_udp");
        let ipv6_only = unpack!("ipv6_only");

        let tcp_keepalive = unpack!("tcp_keepalive", usize);
        let tcp_keepalive_probe = unpack!("tcp_keepalive", usize);
        let tcp_timeout = unpack!("tcp_timeout", usize);
        let udp_timeout = unpack!("udp_timeout", usize);

        let send_proxy = unpack!("send_proxy", bool);
        let send_proxy_version = unpack!("send_proxy_version", usize);

        let accept_proxy = unpack!("accept_proxy", bool);
        let accept_proxy_timeout = unpack!("accept_proxy_timeout", usize);

        Self {
            no_tcp,
            use_udp,
            ipv6_only,
            tcp_keepalive,
            tcp_keepalive_probe,
            tcp_timeout,
            udp_timeout,
            send_proxy,
            accept_proxy,
            send_proxy_version,
            accept_proxy_timeout,
        }
    }
}
