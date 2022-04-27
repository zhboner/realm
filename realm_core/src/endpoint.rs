//! Relay endpoint.

use std::fmt::{Display, Formatter};
use std::net::SocketAddr;

#[cfg(feature = "transport")]
use kaminari::mix::{MixAccept, MixConnect};

/// Remote address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteAddr {
    SocketAddr(SocketAddr),
    DomainName(String, u16),
}

impl Display for RemoteAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use RemoteAddr::*;
        match self {
            SocketAddr(addr) => write!(f, "{}", addr),
            DomainName(host, port) => write!(f, "{}:{}", host, port),
        }
    }
}

/// Connect or associate options.
#[derive(Debug, Clone)]
pub struct ConnectOpts {
    pub connect_timeout: usize,
    pub associate_timeout: usize,
    pub bind_address: Option<SocketAddr>,
    pub bind_interface: Option<String>,

    #[cfg(feature = "proxy-protocol")]
    pub proxy_opts: ProxyOpts,

    #[cfg(feature = "transport")]
    pub transport: Option<(MixAccept, MixConnect)>,
}

/// Relay endpoint.
#[derive(Debug, Clone)]
pub struct Endpoint {
    pub laddr: SocketAddr,
    pub raddr: RemoteAddr,
    pub conn_opts: ConnectOpts,
}

/// Proxy protocol options.
#[allow(unused)]
#[derive(Debug, Default, Clone, Copy)]
pub struct ProxyOpts {
    pub send_proxy: bool,
    pub accept_proxy: bool,
    pub send_proxy_version: usize,
    pub accept_proxy_timeout: usize,
}

#[allow(unused)]
impl ProxyOpts {
    #[inline]
    pub(crate) const fn enabled(&self) -> bool {
        self.send_proxy || self.accept_proxy
    }
}
