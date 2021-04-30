use std::net;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::time::sleep;
use trust_dns_resolver::config::*;
use trust_dns_resolver::TokioAsyncResolver;

fn need_resolve(addr: &str) -> bool {
    addr.parse::<net::IpAddr>().is_err()
}

async fn resolve_single(resolver: &TokioAsyncResolver, addr: &String) -> Option<net::IpAddr> {
    if !need_resolve(&addr) {
        return Some(addr.parse::<net::IpAddr>().unwrap());
    }

    let remote_addr = format!("{}.", addr);
    let res = resolver.lookup_ip(remote_addr).await.unwrap();

    match res.iter().find(|ip| ip.is_ipv4()) {
        Some(ip_v4) => Some(ip_v4),
        None => {
            if let Some(ip_v6) = res.iter().find(|ip| ip.is_ipv6()) {
                Some(ip_v6)
            } else {
                println!("Cannot resolve {}", addr);
                return None;
            }
        }
    }
}

pub async fn resolve(addr_list: Vec<String>, ip_list: Vec<Arc<RwLock<net::IpAddr>>>) {
    let resolver =
        async { TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default()) }
            .await
            .unwrap();
    let cache = "0.0.0.0".parse::<net::IpAddr>().unwrap();
    let size = addr_list.len();
    let mut cache_list = vec![cache.clone(); size];
    loop {
        for (i, addr) in addr_list.iter().enumerate() {
            if let Some(new_ip) = resolve_single(&resolver, addr).await {
                if new_ip != cache_list[i] {
                    cache_list[i] = new_ip;
                    let mut w = ip_list[i].write().unwrap();
                    *w = new_ip;
                    drop(w);
                    println!("Resolved {}: {}", addr, new_ip);
                }
            } else {
                println!("Cannot resolve address {}", addr);
            }
        }

        sleep(Duration::from_secs(60)).await;
    }
}

pub fn print_ips(ip_list: &Vec<Arc<RwLock<std::net::IpAddr>>>) {
    for ip in ip_list {
        println!("{}", ip.read().unwrap());
    }
}
