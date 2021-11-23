use std::io::Result;
use std::fmt::{Formatter, Display};
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
    pub use_udp: bool,
    pub fast_open: bool,
    pub zero_copy: bool,
    pub tcp_timeout: usize,
    pub udp_timeout: usize,
    pub send_through: Option<SocketAddr>,
}

#[derive(Clone)]
pub struct Endpoint {
    pub local: SocketAddr,
    pub remote: RemoteAddr,
    pub opts: ConnectOpts,
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
        use_udp: bool,
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
            local,
            remote,
            opts: ConnectOpts {
                use_udp,
                fast_open,
                zero_copy,
                tcp_timeout,
                udp_timeout,
                send_through: through,
            },
        }
    }
}

impl Display for RemoteAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use RemoteAddr::*;
        match self {
            SocketAddr(x) => write!(f, "{}", x),
            DomainName(addr, port) => write!(f, "{}:{}", addr, port),
        }
    }
}

impl Display for ConnectOpts {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        macro_rules! on_off {
            ($x: expr) => {
                if $x {
                    "on"
                } else {
                    "off"
                }
            };
        }
        if let Some(send_through) = &self.send_through {
            write!(f, "send-through={}, ", send_through)?;
        }
        write!(
            f,
            "udp-forward={}, tcp-fast-open={}, tcp-zero-copy={}, ",
            on_off!(self.use_udp),
            on_off!(self.fast_open),
            on_off!(self.zero_copy)
        )?;
        write!(
            f,
            "tcp-timeout={}s, udp-timeout={}s",
            self.tcp_timeout, self.udp_timeout
        )
    }
}

impl Display for Endpoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} -> {}, options: {}",
            &self.local, &self.remote, &self.opts
        )
    }
}
