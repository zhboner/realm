use std::io::Result;
use std::fmt::{Formatter, Display};
use std::net::SocketAddr;

use crate::dns;
use kaminari::mix::{MixAccept, MixConnect};

#[derive(Clone)]
pub enum RemoteAddr {
    SocketAddr(SocketAddr),
    DomainName(String, u16),
}

#[derive(Clone, Copy, Default)]
pub struct HaproxyOpts {
    pub send_proxy: bool,
    pub accept_proxy: bool,
    pub send_proxy_version: usize,
    pub accept_proxy_timeout: usize,
}

#[derive(Clone, Default)]
pub struct ConnectOpts {
    pub use_udp: bool,
    pub fast_open: bool,
    pub zero_copy: bool,
    pub tcp_timeout: usize,
    pub udp_timeout: usize,
    pub haproxy_opts: HaproxyOpts,
    pub send_through: Option<SocketAddr>,
    pub bind_interface: Option<String>,
    #[cfg(feature = "transport")]
    pub transport: Option<(MixAccept, MixConnect)>,
}

#[derive(Clone)]
pub struct Endpoint {
    pub listen: SocketAddr,
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

impl From<SocketAddr> for RemoteAddr {
    fn from(addr: SocketAddr) -> Self {
        RemoteAddr::SocketAddr(addr)
    }
}

impl Endpoint {
    pub fn new(
        listen: SocketAddr,
        remote: RemoteAddr,
        opts: ConnectOpts,
    ) -> Self {
        Endpoint {
            listen,
            remote,
            opts,
        }
    }
}

// display impl below

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
        const fn on_off(b: bool) -> &'static str {
            if b {
                "on"
            } else {
                "off"
            }
        }

        if let Some(bind_interface) = &self.bind_interface {
            write!(f, "bind-iface={}, ", bind_interface)?;
        }

        if let Some(send_through) = &self.send_through {
            write!(f, "send-through={}; ", send_through)?;
        }
        write!(
            f,
            "udp-forward={}, tcp-fast-open={}, tcp-zero-copy={}; ",
            on_off(self.use_udp),
            on_off(self.fast_open),
            on_off(self.zero_copy)
        )?;

        write!(
            f,
            "send-proxy={0}, send-proxy-version={2}, accept-proxy={1}, accept-proxy-timeout={3}s; ",
            on_off(self.haproxy_opts.send_proxy), on_off(self.haproxy_opts.accept_proxy), self.haproxy_opts.send_proxy_version, self.haproxy_opts.accept_proxy_timeout
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
            "{} -> {}; options: {}",
            &self.listen, &self.remote, &self.opts
        )
    }
}
