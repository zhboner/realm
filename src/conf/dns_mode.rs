use serde::{Serialize, Deserialize};
use trust_dns_resolver::config::LookupIpStrategy;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DnsMode {
    /// Only query for A (Ipv4) records
    Ipv4Only,
    /// Only query for AAAA (Ipv6) records
    Ipv6Only,
    /// Query for A and AAAA in parallel
    Ipv4AndIpv6,
    /// Query for Ipv4 if that fails, query for Ipv6 (default)
    Ipv4ThenIpv6,
    /// Query for Ipv6 if that fails, query for Ipv4
    Ipv6ThenIpv4,
}

impl Default for DnsMode {
    fn default() -> Self {
        Self::Ipv4ThenIpv6
    }
}

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
