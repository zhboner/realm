use cfg_if::cfg_if;
use std::fmt::{Formatter, Display};
use serde::{Serialize, Deserialize};
use super::Config;

cfg_if! {
    if #[cfg(feature = "trust-dns")] {
        use std::net::ToSocketAddrs;
        use trust_dns_resolver as resolver;
        use resolver::config::{LookupIpStrategy, NameServerConfig, Protocol};
        use resolver::config::{ResolverConfig, ResolverOpts};
    }
}

// dns mode
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum DnsMode {
    Ipv4Only,
    Ipv6Only,
    Ipv4AndIpv6,
    Ipv4ThenIpv6,
    Ipv6ThenIpv4,
}

impl Default for DnsMode {
    fn default() -> Self {
        Self::Ipv4AndIpv6
    }
}

impl Display for DnsMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use DnsMode::*;
        let s = match self {
            Ipv4Only => "ipv4_only",
            Ipv6Only => "ipv6_only",
            Ipv4AndIpv6 => "ipv4_and_ipv6",
            Ipv4ThenIpv6 => "ipv4_then_ipv6",
            Ipv6ThenIpv4 => "ipv6_then_ipv4",
        };
        write!(f, "{}", s)
    }
}

impl From<String> for DnsMode {
    fn from(s: String) -> Self {
        use DnsMode::*;
        match s.to_ascii_lowercase().as_str() {
            "ipv4_only" => Ipv4Only,
            "ipv6_only" => Ipv6Only,
            "ipv4_and_ipv6" => Ipv4AndIpv6,
            "ipv4_then_ipv6" => Ipv4ThenIpv6,
            "ipv6_then_ipv4" => Ipv6ThenIpv4,
            _ => Self::default(),
        }
    }
}

#[cfg(feature = "trust-dns")]
impl From<DnsMode> for ResolverOpts {
    fn from(mode: DnsMode) -> Self {
        let ip_strategy = match mode {
            DnsMode::Ipv4Only => LookupIpStrategy::Ipv4Only,
            DnsMode::Ipv6Only => LookupIpStrategy::Ipv6Only,
            DnsMode::Ipv4AndIpv6 => LookupIpStrategy::Ipv4AndIpv6,
            DnsMode::Ipv4ThenIpv6 => LookupIpStrategy::Ipv4thenIpv6,
            DnsMode::Ipv6ThenIpv4 => LookupIpStrategy::Ipv6thenIpv4,
        };
        ResolverOpts {
            ip_strategy,
            ..Default::default()
        }
    }
}

// dns protocol
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum DnsProtocol {
    Tcp,
    Udp,
    TcpAndUdp,
}

impl Default for DnsProtocol {
    fn default() -> Self {
        Self::TcpAndUdp
    }
}

impl Display for DnsProtocol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use DnsProtocol::*;
        let s = match self {
            Tcp => "tcp",
            Udp => "udp",
            TcpAndUdp => "tcp+udp",
        };
        write!(f, "{}", s)
    }
}

impl From<String> for DnsProtocol {
    fn from(s: String) -> Self {
        use DnsProtocol::*;
        match s.to_ascii_lowercase().as_str() {
            "tcp" => Tcp,
            "udp" => Udp,
            _ => TcpAndUdp,
        }
    }
}

#[cfg(feature = "trust-dns")]
impl From<DnsProtocol> for Vec<Protocol> {
    fn from(x: DnsProtocol) -> Self {
        use DnsProtocol::*;
        match x {
            Tcp => vec![Protocol::Tcp],
            Udp => vec![Protocol::Udp],
            TcpAndUdp => vec![Protocol::Tcp, Protocol::Udp],
        }
    }
}

// dns config
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct DnsConf {
    #[serde(default)]
    pub mode: Option<DnsMode>,

    #[serde(default)]
    pub protocol: Option<DnsProtocol>,

    #[serde(default)]
    pub nameservers: Option<Vec<String>>,
}

impl Display for DnsConf {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let DnsConf {
            mode,
            protocol,
            nameservers,
        } = self;

        let mode = match mode {
            Some(m) => *m,
            None => Default::default(),
        };

        let protocol = match protocol {
            Some(x) => *x,
            None => Default::default(),
        };

        let nameservers = match nameservers {
            Some(s) => s.join(", "),
            None => String::from("system"),
        };

        write!(f, "mode={}, protocol={}, ", &mode, &protocol).unwrap();
        write!(f, "servers={}", &nameservers)
    }
}

impl Config for DnsConf {
    type Output = (Option<ResolverConfig>, Option<ResolverOpts>);

    fn resolve(self) -> Self::Output {
        let DnsConf {
            mode,
            protocol,
            nameservers,
        } = self;

        let opts: Option<ResolverOpts> = mode.map(|x| x.into());

        let protocol = protocol.unwrap_or_default();
        if nameservers.is_none() && (protocol == DnsProtocol::default()) {
            return (None, opts);
        }

        let mut conf = ResolverConfig::new();
        let protocols: Vec<Protocol> = protocol.into();
        let nameservers = match nameservers {
            Some(addrs) => addrs
                .iter()
                .map(|x| x.to_socket_addrs().unwrap().next().unwrap())
                .collect(),
            None => {
                use crate::dns::DnsConf as TrustDnsConf;
                let TrustDnsConf { conf, .. } = TrustDnsConf::default();
                let mut addrs: Vec<std::net::SocketAddr> =
                    conf.name_servers().iter().map(|x| x.socket_addr).collect();
                addrs.dedup();
                addrs
            }
        };

        for socket_addr in nameservers {
            for protocol in protocols.clone() {
                conf.add_name_server(NameServerConfig {
                    socket_addr,
                    protocol,
                    tls_dns_name: None,
                    trust_nx_responses: true,
                });
            }
        }

        (Some(conf), opts)
    }
}
