use std::io::{Result, Error, ErrorKind};
use std::net::SocketAddr;
use std::sync::Mutex;

use tokio::runtime::Runtime;

use trust_dns_resolver as resolver;
use resolver::TokioAsyncResolver;
use resolver::config::{ResolverConfig, ResolverOpts};
use resolver::system_conf::read_system_conf;

use lazy_static::lazy_static;

#[derive(Clone)]
pub struct DnsConf {
    pub conf: ResolverConfig,
    pub opts: ResolverOpts,
}

impl Default for DnsConf {
    fn default() -> Self {
        #[cfg(any(unix, windows))]
        let (conf, opts) = read_system_conf().unwrap();

        #[cfg(not(any(unix, windows)))]
        let (conf, opts) = Default::default();

        Self { conf, opts }
    }
}

impl DnsConf {
    pub fn set_conf(&mut self, conf: ResolverConfig) {
        self.conf = conf;
    }

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

pub fn configure(conf: Option<ResolverConfig>, opts: Option<ResolverOpts>) {
    lazy_static::initialize(&DNS_CONF);

    if let Some(conf) = conf {
        DNS_CONF.lock().unwrap().set_conf(conf);
    }
    if let Some(opts) = opts {
        DNS_CONF.lock().unwrap().set_opts(opts);
    }
}

pub fn build() {
    lazy_static::initialize(&DNS);
}

pub async fn resolve(addr: &str, port: u16) -> Result<SocketAddr> {
    DNS.lookup_ip(addr).await.map_or_else(
        |e| Err(Error::new(ErrorKind::Other, e)),
        |ip| Ok(SocketAddr::new(ip.into_iter().next().unwrap(), port)),
    )
}

pub fn resolve_sync(addr: &str, port: u16) -> Result<SocketAddr> {
    let rt = Runtime::new().unwrap();
    rt.block_on(resolve(addr, port))
}
