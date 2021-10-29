use std::io::Result;
use std::net::{SocketAddr, ToSocketAddrs};
use super::dns;

#[derive(Clone)]
pub enum RemoteAddr {
    SocketAddr(SocketAddr),
    DomainName(String, u16),
}

pub struct Endpoint {
    pub udp: bool,
    pub local: SocketAddr,
    pub remote: RemoteAddr,
}

impl RemoteAddr {
    pub async fn into_sockaddr(self) -> Result<SocketAddr> {
        match self {
            Self::SocketAddr(sockaddr) => Ok(sockaddr),
            Self::DomainName(addr, port) => {
                let ip = dns::resolve_async(&addr).await?;
                Ok(SocketAddr::new(ip, port))
            }
        }
    }
    pub async fn to_sockaddr(&self) -> Result<SocketAddr> {
        match self {
            Self::SocketAddr(sockaddr) => Ok(*sockaddr),
            Self::DomainName(addr, port) => {
                let ip = dns::resolve_async(addr).await?;
                Ok(SocketAddr::new(ip, *port))
            }
        }
    }
}

impl Endpoint {
    pub fn new(local: &str, remote: &str, udp: bool) -> Self {
        let local = local
            .to_socket_addrs()
            .expect("invalid local address")
            .next()
            .unwrap();
        let remote = if let Ok(sockaddr) = remote.parse::<SocketAddr>() {
            RemoteAddr::SocketAddr(sockaddr)
        } else {
            let mut iter = remote.rsplitn(2, ':');
            let port = iter.next().unwrap().parse::<u16>().unwrap();
            let addr = iter.next().unwrap().to_string();
            // test addr
            let _ = dns::resolve_sync(&addr).unwrap();
            RemoteAddr::DomainName(addr, port)
        };
        Endpoint { udp, local, remote }
    }
}
