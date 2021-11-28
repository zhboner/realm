use cfg_if::cfg_if;
use std::fmt::{Formatter, Display};
use serde::{Serialize, Deserialize};

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
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
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
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DnsConf {
    #[serde(default)]
    pub mode: DnsMode,

    #[serde(default)]
    pub protocol: DnsProtocol,

    #[serde(default)]
    pub nameservers: Vec<String>,
}

impl Display for DnsConf {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let servers = if self.nameservers.is_empty() {
            String::from("system")
        } else {
            self.nameservers.join(", ")
        };

        write!(f, "mode={}, protocol={}, ", self.mode, self.protocol).unwrap();
        write!(f, "servers={}", &servers)
    }
}

#[cfg(feature = "trust-dns")]
impl From<DnsConf> for (ResolverConfig, ResolverOpts) {
    fn from(conf: DnsConf) -> Self {
        let DnsConf {
            mode,
            protocol,
            nameservers,
        } = conf;

        let opts = mode.into();

        let protocols: Vec<Protocol> = protocol.into();

        let nameservers = if nameservers.is_empty() {
            use crate::dns::DnsConf as XdnsConf;
            let XdnsConf { conf, .. } = XdnsConf::default();
            let mut addrs: Vec<std::net::SocketAddr> =
                conf.name_servers().iter().map(|x| x.socket_addr).collect();
            addrs.dedup();
            addrs
        } else {
            nameservers
                .iter()
                .map(|x| x.to_socket_addrs().unwrap().next().unwrap())
                .collect()
        };

        let mut conf = ResolverConfig::new();

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
        (conf, opts)
    }
}

// compatible dns config
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged, rename_all = "snake_case")]
pub enum CompatibleDnsConf {
    DnsConf(DnsConf),
    DnsMode(DnsMode),
    None,
}

impl Default for CompatibleDnsConf {
    fn default() -> Self {
        Self::None
    }
}

impl AsRef<DnsConf> for CompatibleDnsConf {
    fn as_ref(&self) -> &DnsConf {
        match self {
            CompatibleDnsConf::DnsConf(x) => x,
            _ => unreachable!(),
        }
    }
}

impl AsMut<DnsConf> for CompatibleDnsConf {
    fn as_mut(&mut self) -> &mut DnsConf {
        match self {
            CompatibleDnsConf::DnsConf(x) => x,
            _ => unreachable!(),
        }
    }
}

impl Display for CompatibleDnsConf {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use CompatibleDnsConf::*;
        match self {
            DnsConf(conf) => write!(f, "{}", conf),
            DnsMode(mode) => write!(
                f,
                "{}, protocol={}, servers=system",
                mode,
                DnsProtocol::default(),
            ),
            None => write!(
                f,
                "mode={}, protocol={}, servers=system",
                self::DnsMode::default(),
                DnsProtocol::default(),
            ),
        }
    }
}
