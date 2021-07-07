use std::net::IpAddr;

use tokio::io;
use tokio::runtime::Runtime;
use trust_dns_resolver::TokioAsyncResolver;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use lazy_static::lazy_static;

use crate::utils;

lazy_static! {
    pub static ref DNS: TokioAsyncResolver = TokioAsyncResolver::tokio(
        ResolverConfig::default(),
        ResolverOpts::default()
    )
    .unwrap();
}

pub fn resolve_sync(addr: &str) -> io::Result<IpAddr> {
    let rt = Runtime::new().unwrap();
    rt.block_on(resolve_async(addr))
}

pub async fn resolve_async(addr: &str) -> io::Result<IpAddr> {
    let res = DNS
        .lookup_ip(addr)
        .await
        .map_err(|e| utils::new_io_err(&e.to_string()))?
        .into_iter()
        .next()
        .unwrap();
    Ok(res)
}
