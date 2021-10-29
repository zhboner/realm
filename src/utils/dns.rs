use std::io::{Result, Error, ErrorKind};
use std::net::IpAddr;

use tokio::runtime::Runtime;
use trust_dns_resolver::TokioAsyncResolver;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts, LookupIpStrategy};
use lazy_static::lazy_static;

static mut RESOLVE_STRATEGY: LookupIpStrategy = LookupIpStrategy::Ipv4thenIpv6;

lazy_static! {
    static ref DNS: TokioAsyncResolver = TokioAsyncResolver::tokio(
        ResolverConfig::default(),
        ResolverOpts {
            ip_strategy: unsafe { RESOLVE_STRATEGY },
            ..Default::default()
        }
    )
    .unwrap();
}

pub fn init_resolver(strategy: LookupIpStrategy) {
    unsafe { RESOLVE_STRATEGY = strategy };
    lazy_static::initialize(&DNS);
}

pub fn resolve_sync(addr: &str) -> Result<IpAddr> {
    let rt = Runtime::new().unwrap();
    rt.block_on(resolve_async(addr))
}

pub async fn resolve_async(addr: &str) -> Result<IpAddr> {
    let res = DNS
        .lookup_ip(addr)
        .await
        .map_err(|e| Error::new(ErrorKind::Other, e))?
        .into_iter()
        .next()
        .unwrap();
    Ok(res)
}
