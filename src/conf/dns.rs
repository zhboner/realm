use cfg_if::cfg_if;
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
#[derive(Debug, Serialize, Deserialize)]
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
        Self::Ipv4ThenIpv6
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

// dns config
#[derive(Debug, Serialize, Deserialize)]
pub struct DnsConf {
    #[serde(default)]
    pub mode: DnsMode,

    #[serde(default)]
    pub protocol: String,

    #[serde(default)]
    pub nameservers: Vec<String>,
}

#[cfg(feature = "trust-dns")]
fn read_protocol(net: &str) -> Vec<Protocol> {
    match net.to_ascii_lowercase().as_str() {
        "tcp" => vec![Protocol::Tcp],
        "udp" => vec![Protocol::Udp],
        _ => vec![Protocol::Tcp, Protocol::Udp],
    }
}

#[cfg(feature = "trust-dns")]
impl From<DnsConf> for (ResolverConfig, ResolverOpts) {
    fn from(config: DnsConf) -> Self {
        let opts = config.mode.into();

        let protocols = read_protocol(&config.protocol);

        let nameservers = if config.nameservers.is_empty() {
            use crate::dns::DnsConf as XdnsConf;
            let XdnsConf { conf, .. } = XdnsConf::default();
            let mut addrs: Vec<std::net::SocketAddr> =
                conf.name_servers().iter().map(|x| x.socket_addr).collect();
            addrs.dedup();
            addrs
        } else {
            config
                .nameservers
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
pub enum CompatibeDnsConf {
    Dns(DnsConf),
    DnsMode(DnsMode),
    None,
}

impl Default for CompatibeDnsConf {
    fn default() -> Self {
        Self::None
    }
}
