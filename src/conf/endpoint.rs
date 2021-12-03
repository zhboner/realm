use serde::{Serialize, Deserialize};
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use crate::utils::{Endpoint, RemoteAddr, ConnectOpts};

#[derive(Debug, Serialize, Deserialize)]
pub struct EndpointConf {
    #[serde(default)]
    pub udp: bool,

    #[serde(default)]
    pub fast_open: bool,

    #[serde(default)]
    pub zero_copy: bool,

    #[serde(default = "tcp_timeout")]
    pub tcp_timeout: usize,

    #[serde(default = "udp_timeout")]
    pub udp_timeout: usize,

    pub local: String,

    pub remote: String,

    #[serde(default)]
    pub through: String,
}

const fn tcp_timeout() -> usize {
    crate::utils::TCP_TIMEOUT
}

const fn udp_timeout() -> usize {
    crate::utils::UDP_TIMEOUT
}

impl EndpointConf {
    fn build_local(&self) -> SocketAddr {
        self.local
            .to_socket_addrs()
            .expect("invalid local address")
            .next()
            .unwrap()
    }

    fn build_remote(&self) -> RemoteAddr {
        let Self { remote, .. } = self;
        if let Ok(sockaddr) = remote.parse::<SocketAddr>() {
            RemoteAddr::SocketAddr(sockaddr)
        } else {
            let mut iter = remote.rsplitn(2, ':');
            let port = iter.next().unwrap().parse::<u16>().unwrap();
            let addr = iter.next().unwrap().to_string();
            // test addr
            let _ = crate::dns::resolve_sync(&addr, 0).unwrap();
            RemoteAddr::DomainName(addr, port)
        }
    }

    fn build_send_through(&self) -> Option<SocketAddr> {
        let Self { through, .. } = self;
        match through.to_socket_addrs() {
            Ok(mut x) => Some(x.next().unwrap()),
            Err(_) => {
                let mut ipstr = String::from(through);
                ipstr.retain(|c| c != '[' && c != ']');
                ipstr
                    .parse::<IpAddr>()
                    .map_or(None, |ip| Some(SocketAddr::new(ip, 0)))
            }
        }
    }

    fn build_conn_opts(&self) -> ConnectOpts {
        let Self {
            udp,
            fast_open,
            zero_copy,
            tcp_timeout,
            udp_timeout,
            ..
        } = *self;

        ConnectOpts {
            use_udp: udp,
            fast_open,
            zero_copy,
            tcp_timeout,
            udp_timeout,
            send_through: self.build_send_through(),
        }
    }

    pub fn build(&self) -> Endpoint {
        let local = self.build_local();
        let remote = self.build_remote();
        let opts = self.build_conn_opts();
        Endpoint::new(local, remote, opts)
    }
}
