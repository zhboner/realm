use serde::{Serialize, Deserialize};
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use super::{NetConf, Config};
use crate::utils::{Endpoint, RemoteAddr};

#[derive(Debug, Serialize, Deserialize)]
pub struct EndpointConf {
    pub listen: String,

    pub remote: String,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub through: Option<String>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interface: Option<String>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Config::is_empty")]
    pub network: NetConf,
}

impl EndpointConf {
    fn build_local(&self) -> SocketAddr {
        self.listen
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
        let through = match through {
            Some(x) => x,
            None => return None,
        };
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
}

impl Config for EndpointConf {
    type Output = Endpoint;

    fn is_empty(&self) -> bool {
        false
    }

    fn build(self) -> Self::Output {
        let local = self.build_local();
        let remote = self.build_remote();

        let through = self.build_send_through();
        // iface is untested

        let mut conn_opts = self.network.build();
        conn_opts.send_through = through;
        conn_opts.bind_interface = self.interface;
        Endpoint::new(local, remote, conn_opts)
    }

    fn rst_field(&mut self, _: &Self) -> &mut Self {
        unreachable!()
    }

    fn take_field(&mut self, _: &Self) -> &mut Self {
        unreachable!()
    }

    fn from_cmd_args(matches: &clap::ArgMatches) -> Self {
        let listen = matches.value_of("local").unwrap().to_string();
        let remote = matches.value_of("remote").unwrap().to_string();
        let through = matches.value_of("through").map(String::from);
        let interface = matches.value_of("interface").map(String::from);

        EndpointConf {
            listen,
            remote,
            through,
            interface,
            network: Default::default(),
        }
    }
}
