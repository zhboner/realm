//! Relay endpoint.

use std::fmt::{Display, Formatter};
use std::net::SocketAddr;

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
}

/// Relay endpoint.
#[derive(Debug, Clone)]
pub struct Endpoint {
    pub listen: SocketAddr,
    pub remote: RemoteAddr,
    pub conn_opts: ConnectOpts,
}
