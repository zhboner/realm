//! Global dns resolver.

use std::io::{Result, Error, ErrorKind};
use std::net::SocketAddr;
use std::sync::Mutex;

use trust_dns_resolver as resolver;
use resolver::TokioAsyncResolver;
use resolver::config::{ResolverConfig, ResolverOpts};
use resolver::system_conf::read_system_conf;
use resolver::lookup_ip::{LookupIp, LookupIpIter};

use lazy_static::lazy_static;

use crate::endpoint::RemoteAddr;

/// Dns config.
#[derive(Debug, Clone)]
pub struct DnsConf {
    pub conf: ResolverConfig,
    pub opts: ResolverOpts,
}

/// Use system config on unix(except android) or windows,
/// otherwise use google's public dns servers.
impl Default for DnsConf {
    fn default() -> Self {
        #[cfg(any(all(unix, not(target_os = "android")), windows))]
        let (conf, opts) = read_system_conf().unwrap();

        #[cfg(not(any(all(unix, not(target_os = "android")), windows)))]
        let (conf, opts) = Default::default();

        Self { conf, opts }
    }
}

impl DnsConf {
    /// Set resolver config.
    pub fn set_conf(&mut self, conf: ResolverConfig) {
        self.conf = conf;
    }

    /// Set resolver options.
    pub fn set_opts(&mut self, opts: ResolverOpts) {
        self.opts = opts;
    }
}

lazy_static! {
    static ref DNS_CONF: Mutex<DnsConf> = Mutex::new(DnsConf::default());
    static ref DNS: TokioAsyncResolver = {
        let DnsConf { conf, opts } = DNS_CONF.lock().unwrap().clone();
        TokioAsyncResolver::tokio(conf, opts).unwrap()
    };
}

/// Configure global dns resolver.
pub fn configure(conf: Option<ResolverConfig>, opts: Option<ResolverOpts>) {
    lazy_static::initialize(&DNS_CONF);

    if let Some(conf) = conf {
        DNS_CONF.lock().unwrap().set_conf(conf);
    }
    if let Some(opts) = opts {
        DNS_CONF.lock().unwrap().set_opts(opts);
    }
}

/// Setup global dns resolver.
pub fn build() {
    lazy_static::initialize(&DNS);
}

/// Lookup ip with global dns resolver.
pub async fn resolve_ip(ip: &str) -> Result<LookupIp> {
    DNS.lookup_ip(ip)
        .await
        .map_or_else(|e| Err(Error::new(ErrorKind::Other, e)), Ok)
}

/// Lookup socketaddr with global dns resolver.
pub async fn resolve_addr(addr: &RemoteAddr) -> Result<LookupRemoteAddr<'_>> {
    use RemoteAddr::*;
    use LookupRemoteAddr::*;
    match addr {
        SocketAddr(addr) => Ok(NoLookup(addr)),
        DomainName(ip, port) => {
            resolve_ip(ip).await.map(|ip| Dolookup(ip, *port))
        }
    }
}

/// Resolved result.
pub enum LookupRemoteAddr<'a> {
    NoLookup(&'a SocketAddr),
    Dolookup(LookupIp, u16),
}

impl LookupRemoteAddr<'_> {
    /// Get view of resolved result.
    pub fn iter(&self) -> LookupRemoteAddrIter {
        use LookupRemoteAddr::*;
        match self {
            NoLookup(addr) => {
                LookupRemoteAddrIter::NoLookup(std::iter::once(addr))
            }
            Dolookup(ip, port) => {
                LookupRemoteAddrIter::DoLookup(ip.iter(), *port)
            }
        }
    }
}

/// View of resolved result.
pub enum LookupRemoteAddrIter<'a> {
    NoLookup(std::iter::Once<&'a SocketAddr>),
    DoLookup(LookupIpIter<'a>, u16),
}

impl Iterator for LookupRemoteAddrIter<'_> {
    type Item = SocketAddr;

    fn next(&mut self) -> Option<Self::Item> {
        use LookupRemoteAddrIter::*;
        match self {
            NoLookup(addr) => addr.next().copied(),
            DoLookup(ip, port) => {
                ip.next().map(|ip| SocketAddr::new(ip, *port))
            }
        }
    }
}
