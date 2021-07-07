use std::net::{SocketAddr, ToSocketAddrs};
use tokio::io;
use super::dns;

#[derive(Clone)]
pub enum RemoteAddr {
    SocketAddr(SocketAddr),
    DomainName(String, u16),
}

pub struct Endpoint {
    pub local: SocketAddr,
    pub remote: RemoteAddr,
}

pub fn new_io_err(e: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Other, e)
}

impl RemoteAddr {
    pub async fn to_sockaddr(self) -> io::Result<SocketAddr> {
        match self {
            Self::SocketAddr(sockaddr) => Ok(sockaddr),
            Self::DomainName(addr, port) => {
                let ip = dns::resolve_async(&addr).await?;
                Ok(SocketAddr::new(ip, port))
            }
        }
    }
    pub async fn to_sockaddr_as_ref(&self) -> io::Result<SocketAddr> {
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
    pub fn new(local: &str, remote: &str) -> Self {
        let local = local
            .to_socket_addrs()
            .expect("invalid local address")
            .next()
            .unwrap();
        let remote = if let Ok(mut sockaddr) = remote.to_socket_addrs() {
            RemoteAddr::SocketAddr(sockaddr.next().unwrap())
        } else {
            let mut iter = remote.splitn(2, ':');
            let addr = iter.next().unwrap().to_string();
            let port = iter.next().unwrap().parse::<u16>().unwrap();
            // test addr
            let _ = dns::resolve_sync(&addr).unwrap();
            RemoteAddr::DomainName(addr, port)
        };
        Endpoint { local, remote }
    }
}
