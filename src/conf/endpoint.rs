use serde::{Serialize, Deserialize};
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};

use realm_core::endpoint::{Endpoint, RemoteAddr};

#[cfg(feature = "transport")]
use kaminari::mix::{MixAccept, MixConnect};

use super::{Config, NetConf, NetInfo};

#[derive(Debug, Serialize, Deserialize)]
pub struct EndpointConf {
    pub listen: String,

    pub remote: String,

    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub extra_remotes: Vec<String>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub through: Option<String>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interface: Option<String>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listen_transport: Option<String>,

    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_transport: Option<String>,

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
        Self::build_remote_x(&self.remote)
    }

    fn build_remote_x(remote: &str) -> RemoteAddr {
        if let Ok(sockaddr) = remote.parse::<SocketAddr>() {
            RemoteAddr::SocketAddr(sockaddr)
        } else {
            let mut iter = remote.rsplitn(2, ':');
            let port = iter.next().unwrap().parse::<u16>().unwrap();
            let addr = iter.next().unwrap().to_string();
            // test addr
            remote.to_socket_addrs().unwrap().next().unwrap();
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
                ipstr.parse::<IpAddr>().map_or(None, |ip| Some(SocketAddr::new(ip, 0)))
            }
        }
    }

    #[cfg(feature = "transport")]
    fn build_transport(&self) -> Option<(MixAccept, MixConnect)> {
        use kaminari::mix::{MixClientConf, MixServerConf};
        use kaminari::opt::get_ws_conf;
        use kaminari::opt::get_tls_client_conf;
        use kaminari::opt::get_tls_server_conf;

        let Self {
            listen_transport,
            remote_transport,
            ..
        } = self;

        let listen_ws = listen_transport.as_ref().and_then(|s| get_ws_conf(s));
        let listen_tls = listen_transport.as_ref().and_then(|s| get_tls_server_conf(s));

        let remote_ws = remote_transport.as_ref().and_then(|s| get_ws_conf(s));
        let remote_tls = remote_transport.as_ref().and_then(|s| get_tls_client_conf(s));

        if matches!(
            (&listen_ws, &listen_tls, &remote_ws, &remote_tls),
            (None, None, None, None)
        ) {
            None
        } else {
            let ac = MixAccept::new_shared(MixServerConf {
                ws: listen_ws,
                tls: listen_tls,
            });
            let cc = MixConnect::new_shared(MixClientConf {
                ws: remote_ws,
                tls: remote_tls,
            });
            Some((ac, cc))
        }
    }
}

#[derive(Debug)]
pub struct EndpointInfo {
    pub no_tcp: bool,
    pub use_udp: bool,
    pub endpoint: Endpoint,
}

impl Config for EndpointConf {
    type Output = EndpointInfo;

    fn is_empty(&self) -> bool {
        false
    }

    fn build(self) -> Self::Output {
        let laddr = self.build_local();
        let raddr = self.build_remote();

        let extra_raddrs = self.extra_remotes.iter().map(|r| Self::build_remote_x(r)).collect();

        // build partial conn_opts from netconf
        let NetInfo {
            mut conn_opts,
            no_tcp,
            use_udp,
        } = self.network.build();

        // build left fields of conn_opts

        conn_opts.bind_address = self.build_send_through();

        #[cfg(feature = "transport")]
        {
            conn_opts.transport = self.build_transport();
        }

        conn_opts.bind_interface = self.interface;

        EndpointInfo {
            no_tcp,
            use_udp,
            endpoint: Endpoint {
                laddr,
                raddr,
                conn_opts,
                extra_raddrs,
            },
        }
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
        let listen_transport = matches.value_of("listen_transport").map(String::from);
        let remote_transport = matches.value_of("remote_transport").map(String::from);

        EndpointConf {
            listen,
            remote,
            through,
            interface,
            listen_transport,
            remote_transport,
            network: Default::default(),
            extra_remotes: Vec::new(),
        }
    }
}
