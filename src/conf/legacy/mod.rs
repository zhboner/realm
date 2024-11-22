use serde::{Serialize, Deserialize};

use super::{FullConf, EndpointConf};

// from https://github.com/zhboner/realm/blob/8ad8f0405e97cc470ba8b76c059c203b7381d2fb/src/lib.rs#L58-L63
// pub struct ConfigFile {
//     pub listening_addresses: Vec<String>,
//     pub listening_ports: Vec<String>,
//     pub remote_addresses: Vec<String>,
//     pub remote_ports: Vec<String>,
// }
#[derive(Serialize, Deserialize)]
pub struct LegacyConf {
    #[serde(rename = "listening_addresses")]
    pub listen_addrs: Vec<String>,
    #[serde(rename = "listening_ports")]
    pub listen_ports: Vec<String>,
    #[serde(rename = "remote_addresses")]
    pub remote_addrs: Vec<String>,
    #[serde(rename = "remote_ports")]
    pub remote_ports: Vec<String>,
}

fn flatten_ports(ports: Vec<String>) -> Vec<u16> {
    ports
        .into_iter()
        .flat_map(|range| match (range.split('-').next(), range.split('-').nth(1)) {
            (Some(start), Some(end)) => start.parse::<u16>().unwrap()..end.parse::<u16>().unwrap() + 1,
            (Some(start), None) => start.parse::<u16>().unwrap()..start.parse::<u16>().unwrap() + 1,
            _ => panic!("failed to parse ports"),
        })
        .collect()
}

fn join_addr_port(addrs: Vec<String>, ports: Vec<u16>, len: usize) -> Vec<String> {
    use std::iter::repeat;

    let port0 = ports[0];
    let addr0 = addrs[0].clone();

    let port_iter = ports.into_iter().take(len).chain(repeat(port0)).take(len);
    let addr_iter = addrs.into_iter().take(len).chain(repeat(addr0)).take(len);

    addr_iter
        .zip(port_iter)
        .map(|(addr, port)| format!("{}:{}", addr, port))
        .collect()
}

impl From<LegacyConf> for FullConf {
    fn from(x: LegacyConf) -> Self {
        let LegacyConf {
            listen_addrs,
            listen_ports,
            remote_addrs,
            remote_ports,
        } = x;

        let listen_ports = flatten_ports(listen_ports);
        let remote_ports = flatten_ports(remote_ports);

        let len = listen_ports.len();

        let listen = join_addr_port(listen_addrs, listen_ports, len);
        let remote = join_addr_port(remote_addrs, remote_ports, len);

        let endpoints = listen
            .into_iter()
            .zip(remote)
            .map(|(listen, remote)| EndpointConf {
                listen,
                remote,
                through: None,
                interface: None,
                listen_interface: None,
                listen_transport: None,
                remote_transport: None,
                network: Default::default(),
                extra_remotes: Vec::new(),
                balance: None,
            })
            .collect();

        FullConf {
            endpoints,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    macro_rules! strvec {
        ( $( $x: expr ),+ ) => {
            vec![
                $(
                    String::from($x),
                )+
            ]
        };
    }

    #[test]
    fn flatten_ports() {
        let v1 = strvec!["1-4"];
        let v2 = strvec!["1-2", "3-4"];
        let v3 = strvec!["1-3", "4"];
        let v4 = strvec!["1", "2", "3", "4"];
        assert_eq!(super::flatten_ports(v1), [1, 2, 3, 4]);
        assert_eq!(super::flatten_ports(v2), [1, 2, 3, 4]);
        assert_eq!(super::flatten_ports(v3), [1, 2, 3, 4]);
        assert_eq!(super::flatten_ports(v4), [1, 2, 3, 4]);
    }

    #[test]
    fn join_addr_port() {
        let addrs = strvec!["a.com", "b.com", "c.com"];
        let ports = vec![1, 2, 3];
        let result = vec!["a.com:1", "b.com:2", "c.com:3"];
        assert_eq!(super::join_addr_port(addrs, ports, 3), result);

        let addrs = strvec!["a.com", "b.com", "c.com"];
        let ports = vec![1, 2, 3];
        let result = vec!["a.com:1", "b.com:2", "c.com:3"];
        assert_eq!(super::join_addr_port(addrs, ports, 2), result[..2]);

        let addrs = strvec!["a.com", "b.com", "c.com"];
        let ports = vec![1, 2, 3];
        let result = vec!["a.com:1", "b.com:2", "c.com:3", "a.com:1"];
        assert_eq!(super::join_addr_port(addrs, ports, 4), result);

        let addrs = strvec!["a.com", "b.com", "c.com"];
        let ports = vec![1, 2, 3, 4, 5, 6];
        let result = vec!["a.com:1", "b.com:2", "c.com:3", "a.com:4"];
        assert_eq!(super::join_addr_port(addrs, ports, 4), result);

        let addrs = strvec!["a.com", "b.com", "c.com", "d.com", "e.com"];
        let ports = vec![1, 2, 3];
        let result = vec!["a.com:1", "b.com:2", "c.com:3", "d.com:1"];
        assert_eq!(super::join_addr_port(addrs, ports, 4), result);

        let addrs = strvec!["a.com", "b.com", "c.com"];
        let ports = vec![1, 2, 3];
        let result = vec!["a.com:1", "b.com:2", "c.com:3", "a.com:1", "a.com:1"];
        assert_eq!(super::join_addr_port(addrs, ports, 5), result);
    }
}
