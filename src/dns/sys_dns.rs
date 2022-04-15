use std::io::Result;
use std::net::{SocketAddr, ToSocketAddrs};

pub fn resolve(addr: &str, port: u16) -> Result<SocketAddr> {
    (addr, port)
        .to_socket_addrs()
        .map(|mut x| x.next().unwrap())
}
