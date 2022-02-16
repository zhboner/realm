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
impl From<DnsMode> for LookupIpStrategy {
    fn from(mode: DnsMode) -> Self {
        match mode {
            DnsMode::Ipv4Only => LookupIpStrategy::Ipv4Only,
            DnsMode::Ipv6Only => LookupIpStrategy::Ipv6Only,
            DnsMode::Ipv4AndIpv6 => LookupIpStrategy::Ipv4AndIpv6,
            DnsMode::Ipv4ThenIpv6 => LookupIpStrategy::Ipv4thenIpv6,
            DnsMode::Ipv6ThenIpv4 => LookupIpStrategy::Ipv6thenIpv4,
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
    // ResolverOpts
    #[serde(default)]
    pub mode: Option<DnsMode>,

    // MAX_TTL: u32 = 86400_u32
    // https://docs.rs/trust-dns-resolver/latest/src/trust_dns_resolver/dns_lru.rs.html#26
    #[serde(default)]
    pub min_ttl: Option<u32>,

    #[serde(default)]
    pub max_ttl: Option<u32>,

    #[serde(default)]
    pub cache_size: Option<usize>,

    // ResolverConfig
    #[serde(default)]
    pub protocol: Option<DnsProtocol>,

    #[serde(default)]
    pub nameservers: Option<Vec<String>>,
}

impl Display for DnsConf {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        macro_rules! default {
            ($ref: expr) => {
                match $ref {
                    Some(x) => *x,
                    None => Default::default(),
                }
            };
            ($ref: expr, $value: expr) => {
                match $ref {
                    Some(x) => *x,
                    None => $value,
                }
            };
        }
        let DnsConf {
            mode,
            min_ttl,
            max_ttl,
            cache_size,
            protocol,
            nameservers,
        } = self;

        let mode = default!(mode);

        let min_ttl = default!(min_ttl, 0_u32);

        let max_ttl = default!(max_ttl, 86400_u32);

        let cache_size = default!(cache_size, 32_usize);

        let protocol = default!(protocol);

        let nameservers = match nameservers {
            Some(s) => s.join(", "),
            None => String::from("system"),
        };

        write!(f, "mode={}, protocol={}, ", &mode, &protocol).unwrap();
        write!(
            f,
            "min-ttl={}, max-ttl={}, cache-size={}, ",
            min_ttl, max_ttl, cache_size
        )
        .unwrap();
        write!(f, "servers={}", &nameservers)
    }
}

impl Config for DnsConf {
    #[cfg(feature = "trust-dns")]
    type Output = (Option<ResolverConfig>, Option<ResolverOpts>);

    #[cfg(not(feature = "trust-dns"))]
    type Output = ();

    #[cfg(not(feature = "trust-dns"))]
    fn build(self) -> Self::Output {
        unreachable!()
    }

    #[cfg(feature = "trust-dns")]
    fn build(self) -> Self::Output {
        use std::time::Duration;

        let DnsConf {
            mode,
            protocol,
            nameservers,
            min_ttl,
            max_ttl,
            cache_size,
        } = self;

        // parse into ResolverOpts
        // default value:
        // https://docs.rs/trust-dns-resolver/latest/src/trust_dns_resolver/config.rs.html#681-737

        macro_rules! all_none {
            ( $( $x: expr ),* ) => {{
                let mut res = true;
                $(
                    res = res && $x.is_none();
                )*
                res
            }}
        }

        let opts = if all_none![mode, min_ttl, max_ttl, cache_size] {
            None
        } else {
            let ip_strategy: LookupIpStrategy =
                mode.map(|x| x.into()).unwrap_or_default();

            let positive_min_ttl =
                min_ttl.map(|x| Duration::from_secs(x as u64));

            let positive_max_ttl =
                max_ttl.map(|x| Duration::from_secs(x as u64));

            let cache_size = cache_size.unwrap_or({
                let ResolverOpts { cache_size, .. } = Default::default();
                cache_size
            });

            Some(ResolverOpts {
                ip_strategy,
                positive_min_ttl,
                positive_max_ttl,
                cache_size,
                ..Default::default()
            })
        };

        // parse into ResolverConfig
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

    fn rst_field(&mut self, other: &Self) -> &mut Self {
        use crate::rst;
        let other = other.clone();
        rst!(self, mode, other);
        rst!(self, min_ttl, other);
        rst!(self, max_ttl, other);
        rst!(self, cache_size, other);
        rst!(self, protocol, other);
        rst!(self, nameservers, other);
        self
    }

    fn take_field(&mut self, other: &Self) -> &mut Self {
        use crate::take;
        let other = other.clone();
        take!(self, mode, other);
        take!(self, min_ttl, other);
        take!(self, max_ttl, other);
        take!(self, cache_size, other);
        take!(self, protocol, other);
        take!(self, nameservers, other);
        self
    }

    fn from_cmd_args(matches: &clap::ArgMatches) -> Self {
        let mode = matches.value_of("dns_mode").map(|x| String::from(x).into());

        let min_ttl = matches
            .value_of("dns_min_ttl")
            .map(|x| x.parse::<u32>().unwrap());

        let max_ttl = matches
            .value_of("dns_max_ttl")
            .map(|x| x.parse::<u32>().unwrap());

        let cache_size = matches
            .value_of("dns_cache_size")
            .map(|x| x.parse::<usize>().unwrap());

        let protocol = matches
            .value_of("dns_protocol")
            .map(|x| String::from(x).into());

        let nameservers = matches
            .value_of("dns_servers")
            .map(|x| x.split(',').map(String::from).collect());

        Self {
            mode,
            min_ttl,
            max_ttl,
            cache_size,
            protocol,
            nameservers,
        }
    }
}
