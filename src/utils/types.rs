use std::io::Result;
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};

use crate::dns;

pub const TCP_TIMEOUT: usize = 300;
pub const UDP_TIMEOUT: usize = 30;

#[derive(Clone)]
pub enum RemoteAddr {
    SocketAddr(SocketAddr),
    DomainName(String, u16),
}

#[derive(Clone, Copy)]
pub struct ConnectOpts {
    pub fast_open: bool,
    pub zero_copy: bool,
    pub tcp_timeout: usize,
    pub udp_timeout: usize,
    pub send_through: Option<SocketAddr>,
}

#[derive(Clone)]
pub struct Endpoint {
    pub udp: bool,
    pub local: SocketAddr,
    pub remote: RemoteAddr,
    pub conn_opts: ConnectOpts,
}

impl RemoteAddr {
    pub async fn into_sockaddr(self) -> Result<SocketAddr> {
        match self {
            Self::SocketAddr(sockaddr) => Ok(sockaddr),
            Self::DomainName(addr, port) => {
                #[cfg(feature = "trust-dns")]
                {
                    dns::resolve(&addr, port).await
                }

                #[cfg(not(feature = "trust-dns"))]
                {
                    dns::resolve(&addr, port)
                }
            }
        }
    }
    #[allow(unused)]
    pub async fn to_sockaddr(&self) -> Result<SocketAddr> {
        match self {
            Self::SocketAddr(sockaddr) => Ok(*sockaddr),
            Self::DomainName(addr, port) => {
                #[cfg(feature = "trust-dns")]
                {
                    dns::resolve(addr, *port).await
                }

                #[cfg(not(feature = "trust-dns"))]
                {
                    dns::resolve(addr, *port)
                }
            }
        }
    }
}

impl Endpoint {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        local: &str,
        remote: &str,
        through: &str,
        udp: bool,
        fast_open: bool,
        zero_copy: bool,
        tcp_timeout: usize,
        udp_timeout: usize,
    ) -> Self {
        // check local addr
        let local = local
            .to_socket_addrs()
            .expect("invalid local address")
            .next()
            .unwrap();

        // check remote addr
        let remote = if let Ok(sockaddr) = remote.parse::<SocketAddr>() {
            RemoteAddr::SocketAddr(sockaddr)
        } else {
            let mut iter = remote.rsplitn(2, ':');
            let port = iter.next().unwrap().parse::<u16>().unwrap();
            let addr = iter.next().unwrap().to_string();
            // test addr
            let _ = dns::resolve_sync(&addr, 0).unwrap();
            RemoteAddr::DomainName(addr, port)
        };

        // check bind addr
        let through = match through.to_socket_addrs() {
            Ok(mut x) => Some(x.next().unwrap()),
            Err(_) => {
                let mut ipstr = String::from(through);
                ipstr.retain(|c| c != '[' && c != ']');
                ipstr
                    .parse::<IpAddr>()
                    .map_or(None, |ip| Some(SocketAddr::new(ip, 0)))
            }
        };

        Endpoint {
            udp,
            local,
            remote,
            conn_opts: ConnectOpts {
                fast_open,
                zero_copy,
                tcp_timeout,
                udp_timeout,
                send_through: through,
            },
        }
    }
}
