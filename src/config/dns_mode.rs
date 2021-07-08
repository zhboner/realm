use serde::{Serialize, Deserialize};
use trust_dns_resolver::config::LookupIpStrategy;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DnsMode {
    /// Only query for A (Ipv4) records
    Ipv4Only,
    /// Only query for AAAA (Ipv6) records
    Ipv6Only,
    /// Query for A and AAAA in parallel
    Ipv4AndIpv6,
    /// Query for Ipv4 if that fails, query for Ipv6 (default)
    Ipv4thenIpv6,
    /// Query for Ipv6 if that fails, query for Ipv4
    Ipv6thenIpv4,
}

impl Default for DnsMode {
    fn default() -> Self {
        Self::Ipv4thenIpv6
    }
}

impl DnsMode {
    pub fn to_strategy(self) -> LookupIpStrategy {
        match self {
            Self::Ipv4Only => LookupIpStrategy::Ipv4Only,
            Self::Ipv6Only => LookupIpStrategy::Ipv6Only,
            Self::Ipv4AndIpv6 => LookupIpStrategy::Ipv4AndIpv6,
            Self::Ipv4thenIpv6 => LookupIpStrategy::Ipv4thenIpv6,
            Self::Ipv6thenIpv4 => LookupIpStrategy::Ipv6thenIpv4,
        }
    }
}
