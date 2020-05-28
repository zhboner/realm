use std::net;
use std::thread::sleep;
use std::time::Duration;
use std::sync::{RwLock, Arc};
use trust_dns_resolver::config::*;
use trust_dns_resolver::Resolver;

fn need_resolve(addr: &str) -> bool {
    addr.parse::<net::IpAddr>().is_err()
}

pub fn dns_resolve(addr: String, ip: Arc<RwLock::<net::IpAddr>>) {
    let mut cache = "0.0.0.0".parse::<net::IpAddr>().unwrap();

    if !need_resolve(&addr) {
        let mut w = ip.write().unwrap();
        *w = addr.parse::<net::IpAddr>().unwrap();
        return;
    }

    let resolver = Resolver::new(ResolverConfig::default(), ResolverOpts::default()).unwrap();
    loop {
        let remote_addr = format!("{}.", addr);
        let res = resolver.lookup_ip(&remote_addr).unwrap();

        match res.iter().find(|ip| ip.is_ipv4()) {
            Some(ip_v4) => {
                if cache != ip_v4 {
                    cache = ip_v4;
                    let mut w = ip.write().unwrap();
                    *w = ip_v4;
                }
            }
            None => {
                if let Some(ip_v6) = res.iter().find(|ip| ip.is_ipv6()) {
                    if cache != ip_v6 {
                        cache = ip_v6;
                        let mut w = ip.write().unwrap();
                        *w = ip_v6;
                    }
                } else {
                    println!("Cannot resolve {}", addr);
                    return;
                }
            }
        }

        sleep(Duration::from_secs(60));
    }
}
