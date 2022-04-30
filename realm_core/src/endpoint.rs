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

/// Proxy protocol options.
#[cfg(feature = "proxy")]
#[derive(Debug, Default, Clone, Copy)]
pub struct ProxyOpts {
    pub send_proxy: bool,
    pub accept_proxy: bool,
    pub send_proxy_version: usize,
    pub accept_proxy_timeout: usize,
}

#[cfg(feature = "proxy")]
impl ProxyOpts {
    #[inline]
    pub(crate) const fn enabled(&self) -> bool {
        self.send_proxy || self.accept_proxy
    }
}

/// Connect or associate options.
#[derive(Debug, Default, Clone)]
pub struct ConnectOpts {
    pub connect_timeout: usize,
    pub associate_timeout: usize,
    pub bind_address: Option<SocketAddr>,
    pub bind_interface: Option<String>,

    #[cfg(feature = "proxy")]
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

    #[cfg(feature = "multi-remote")]
    pub extra_raddrs: Vec<RemoteAddr>,
}

// display impl below

impl Display for RemoteAddr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use RemoteAddr::*;
        match self {
            SocketAddr(addr) => write!(f, "{}", addr),
            DomainName(host, port) => write!(f, "{}:{}", host, port),
        }
    }
}

impl Display for Endpoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        #[cfg(not(feature = "multi-remote"))]
        write!(f, "{} -> {}; options: {}", &self.laddr, &self.raddr, &self.conn_opts)?;

        #[cfg(feature = "multi-remote")]
        {
            write!(f, "{} -> [{}", &self.laddr, &self.raddr)?;
            for raddr in self.extra_raddrs.iter() {
                write!(f, "|{}", raddr)?;
            }
            write!(f, "]; options: {}", &self.conn_opts)?;
        }

        Ok(())
    }
}

impl Display for ConnectOpts {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let ConnectOpts {
            connect_timeout,
            associate_timeout,
            bind_address,
            bind_interface,

            #[cfg(feature = "proxy")]
            proxy_opts,

            #[cfg(feature = "transport")]
            transport,
        } = self;

        if let Some(iface) = bind_interface {
            write!(f, "bind-iface={}, ", iface)?;
        }

        if let Some(send_through) = bind_address {
            write!(f, "send-through={}; ", send_through)?;
        }

        #[cfg(feature = "proxy")]
        {
            let ProxyOpts {
                send_proxy,
                accept_proxy,
                send_proxy_version,
                accept_proxy_timeout,
            } = proxy_opts;
            write!(
                f,
                "send-proxy={0}, send-proxy-version={2}, accept-proxy={1}, accept-proxy-timeout={3}s; ",
                send_proxy, accept_proxy, send_proxy_version, accept_proxy_timeout
            )?;
        }

        write!(
            f,
            "connect-timeout={}s, associate-timeout={}s; ",
            connect_timeout, associate_timeout
        )?;

        #[cfg(feature = "transport")]
        if let Some((ac, cc)) = transport {
            write!(f, "transport={}||{}", ac, cc)?
        }

        Ok(())
    }
}
