use std::io::{Error, Result};
use std::cell::OnceCell;
use std::net::{SocketAddr, IpAddr, Ipv4Addr, Ipv6Addr};

use log::{info, debug};
use bytes::{Buf, BytesMut};

use proxy_protocol::ProxyHeader;
use proxy_protocol::{version1 as v1, version2 as v2};
use proxy_protocol::{encode, parse};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::endpoint::ProxyOpts;
use crate::time::timeoutfut;

// TODO: replace the "proxy-protocol" crate, and then avoid heap allocation.

// client -> relay -> server
pub async fn handle_proxy(src: &mut TcpStream, dst: &mut TcpStream, opts: ProxyOpts) -> Result<()> {
    let ProxyOpts {
        send_proxy,
        accept_proxy,
        send_proxy_version,
        accept_proxy_timeout,
    } = opts;

    let client_addr = OnceCell::<SocketAddr>::new();
    let server_addr = OnceCell::<SocketAddr>::new();

    // parse PROXY header from client and write log
    // may not get src and dst addr
    if accept_proxy {
        let mut buf = BytesMut::with_capacity(256);
        buf.resize(256, 0);

        // FIXME: may not read the entire header

        // The receiver may apply a short timeout and decide to
        // abort the connection if the protocol header is not seen
        // within a few seconds (at least 3 seconds to cover a TCP retransmit).
        let peek_n = timeoutfut(src.peek(&mut buf), accept_proxy_timeout).await??;

        buf.truncate(peek_n);
        debug!("[tcp]peek initial {} bytes: {:#x}", peek_n, buf);

        let mut slice = buf.as_ref();

        // slice is advanced
        let header = parse(&mut slice).map_err(Error::other)?;
        let parsed_n = peek_n - slice.remaining();
        debug!("[tcp]proxy-protocol parsed, {} bytes", parsed_n);

        // handle parsed header, and print log
        if let Some((src, dst)) = handle_header(header) {
            client_addr.set(src).unwrap();
            server_addr.set(dst).unwrap();
        }

        // header has been parsed, remove these bytes from sock buffer.
        buf.truncate(parsed_n);
        src.read_exact(&mut buf).await?;

        // do not send header to server
        if !send_proxy {
            return Ok(());
        }
    }

    // use real addr
    if !client_addr.get().is_some() {
        client_addr.set(src.peer_addr()?).unwrap();
        // FIXME: what is the dst addr here? seems not defined in the doc
        // the doc only mentions that this field is similar to X-Origin-To
        // which is seldom used
        server_addr
            .set(match client_addr.get().unwrap() {
                SocketAddr::V4(_) => SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
                SocketAddr::V6(_) => SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)), 0),
            })
            .unwrap();
    }

    // Safety: sockaddr is always initialized
    // either parse from PROXY header or use real addr
    let client_addr = client_addr.into_inner().unwrap();
    let server_addr = server_addr.into_inner().unwrap();

    // write header
    let header = encode(make_header(client_addr, server_addr, send_proxy_version)).map_err(Error::other)?;
    debug!("[tcp]send initial {} bytes: {:#x}", header.len(), &header);
    dst.write_all(&header).await?;

    Ok(())
}

macro_rules! unpack {
    ($addr: expr, sin4) => {
        match $addr {
            SocketAddr::V4(x) => x,
            _ => unreachable!(),
        }
    };
    ($addr: expr, sin6) => {
        match $addr {
            SocketAddr::V6(x) => x,
            _ => unreachable!(),
        }
    };
}

fn make_header(client_addr: SocketAddr, server_addr: SocketAddr, send_proxy_version: usize) -> ProxyHeader {
    match send_proxy_version {
        2 => make_header_v2(client_addr, server_addr),
        1 => make_header_v1(client_addr, server_addr),
        _ => unreachable!(),
    }
}

fn make_header_v1(client_addr: SocketAddr, server_addr: SocketAddr) -> ProxyHeader {
    debug!("[tcp]send proxy-protocol-v1: {} => {}", client_addr, server_addr);

    if client_addr.is_ipv4() {
        ProxyHeader::Version1 {
            addresses: v1::ProxyAddresses::Ipv4 {
                source: unpack!(client_addr, sin4),
                destination: unpack!(server_addr, sin4),
            },
        }
    } else {
        ProxyHeader::Version1 {
            addresses: v1::ProxyAddresses::Ipv6 {
                source: unpack!(client_addr, sin6),
                destination: unpack!(server_addr, sin6),
            },
        }
    }
}

fn make_header_v2(client_addr: SocketAddr, server_addr: SocketAddr) -> ProxyHeader {
    debug!("[tcp]send proxy-protocol-v2: {} => {}", client_addr, server_addr);

    ProxyHeader::Version2 {
        command: v2::ProxyCommand::Proxy,
        transport_protocol: v2::ProxyTransportProtocol::Stream,
        addresses: if client_addr.is_ipv4() {
            v2::ProxyAddresses::Ipv4 {
                source: unpack!(client_addr, sin4),
                destination: unpack!(server_addr, sin4),
            }
        } else {
            v2::ProxyAddresses::Ipv6 {
                source: unpack!(client_addr, sin6),
                destination: unpack!(server_addr, sin6),
            }
        },
    }
}

fn handle_header(header: ProxyHeader) -> Option<(SocketAddr, SocketAddr)> {
    use ProxyHeader::{Version1, Version2};
    match header {
        Version1 { addresses } => handle_header_v1(addresses),
        Version2 {
            command,
            transport_protocol,
            addresses,
        } => handle_header_v2(command, transport_protocol, addresses),
        _ => {
            info!("[tcp]accept proxy-protocol-v?");
            None
        }
    }
}

fn handle_header_v1(addr: v1::ProxyAddresses) -> Option<(SocketAddr, SocketAddr)> {
    use v1::ProxyAddresses::*;
    match addr {
        Unknown => {
            info!("[tcp]accept proxy-protocol-v1: unknown");
            None
        }
        Ipv4 { source, destination } => {
            info!("[tcp]accept proxy-protocol-v1: {} => {}", source, destination);
            Some((SocketAddr::V4(source), SocketAddr::V4(destination)))
        }
        Ipv6 { source, destination } => {
            info!("[tcp]accept proxy-protocol-v1: {} => {}", source, destination);
            Some((SocketAddr::V6(source), SocketAddr::V6(destination)))
        }
    }
}

fn handle_header_v2(
    cmd: v2::ProxyCommand,
    proto: v2::ProxyTransportProtocol,
    addr: v2::ProxyAddresses,
) -> Option<(SocketAddr, SocketAddr)> {
    use v2::ProxyCommand as Command;
    use v2::ProxyAddresses as Address;
    use v2::ProxyTransportProtocol as Protocol;

    // The connection endpoints are the sender and the receiver.
    // Such connections exist when the proxy sends health-checks to the server.
    // The receiver must accept this connection as valid and must use the
    // real connection endpoints and discard the protocol block including the
    // family which is ignored
    if let Command::Local = cmd {
        info!("[tcp]accept proxy-protocol-v2: command = LOCAL, ignore");
        return None;
    }

    // only get tcp address
    match proto {
        Protocol::Stream => {}
        Protocol::Unspec => {
            info!("[tcp]accept proxy-protocol-v2: protocol = UNSPEC, ignore");
            return None;
        }
        Protocol::Datagram => {
            info!("[tcp]accept proxy-protocol-v2: protocol = DGRAM, ignore");
            return None;
        }
    }

    match addr {
        Address::Ipv4 { source, destination } => {
            info!("[tcp]accept proxy-protocol-v2: {} => {}", source, destination);
            Some((SocketAddr::V4(source), SocketAddr::V4(destination)))
        }
        Address::Ipv6 { source, destination } => {
            info!("[tcp]accept proxy-protocol-v2: {} => {}", source, destination);
            Some((SocketAddr::V6(source), SocketAddr::V6(destination)))
        }
        Address::Unspec => {
            info!("[tcp]accept proxy-protocol-v2: af_family = AF_UNSPEC, ignore");
            None
        }
        Address::Unix { .. } => {
            info!("[tcp]accept proxy-protocol-v2: af_family = AF_UNIX, ignore");
            None
        }
    }
}
