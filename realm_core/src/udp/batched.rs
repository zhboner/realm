use std::io::Result;
use std::net::SocketAddr;
use tokio::net::UdpSocket;

pub const PACKET_SIZE: usize = 1500;
pub const MAX_PACKETS: usize = 128;

#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SockAddrStore {
    #[cfg(all(target_os = "linux", feature = "batched-udp"))]
    inner: realm_io::mmsg::SockAddrStore,

    #[cfg(not(all(target_os = "linux", feature = "batched-udp")))]
    inner: std::net::SocketAddr,
}

impl SockAddrStore {
    pub const fn new() -> Self {
        Self {
            #[cfg(all(target_os = "linux", feature = "batched-udp"))]
            inner: realm_io::mmsg::SockAddrStore::new(),

            #[cfg(not(all(target_os = "linux", feature = "batched-udp")))]
            inner: {
                use std::net::{IpAddr, Ipv4Addr};
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0)
            },
        }
    }
}

impl Default for SockAddrStore {
    fn default() -> Self {
        Self::new()
    }
}

impl From<SocketAddr> for SockAddrStore {
    fn from(value: SocketAddr) -> Self {
        SockAddrStore { inner: value.into() }
    }
}

impl From<SockAddrStore> for SocketAddr {
    fn from(value: SockAddrStore) -> Self {
        value.inner.into()
    }
}

#[derive(Debug, Clone)]
pub struct Packet {
    pub(super) buf: [u8; PACKET_SIZE],
    pub(super) addr: SockAddrStore,
    pub(super) cursor: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct PacketRef<'buf, 'addr> {
    buf: &'buf [u8],
    addr: &'addr SockAddrStore,
}

impl Packet {
    pub const fn new() -> Self {
        Self {
            buf: [0u8; PACKET_SIZE],
            addr: SockAddrStore::new(),
            cursor: 0u16,
        }
    }

    pub fn ref_with_addr<'a>(&self, addr: &'a SockAddrStore) -> PacketRef<'_, 'a> {
        PacketRef {
            buf: &self.buf[..self.cursor as usize],
            addr,
        }
    }
}

#[cfg(not(all(target_os = "linux", feature = "batched-udp")))]
pub use common::{recv_some, send_all};
#[cfg(not(all(target_os = "linux", feature = "batched-udp")))]
mod common {
    use super::*;
    pub async fn recv_some(sock: &UdpSocket, pkts: &mut [Packet]) -> Result<usize> {
        debug_assert!(!pkts.is_empty());
        let pkt = &mut pkts[0];
        let (bytes, addr) = sock.recv_from(&mut pkt.buf).await?;
        pkt.addr.inner = addr;
        pkt.cursor = bytes as u16;
        Ok(1)
    }

    pub async fn send_all<'a, 'b, I>(sock: &UdpSocket, pkts: I) -> Result<()>
    where
        I: ExactSizeIterator<Item = PacketRef<'a, 'b>>,
    {
        for pkt in pkts {
            let _ = sock.send_to(pkt.buf, &pkt.addr.inner).await?;
        }
        Ok(())
    }
}

#[cfg(all(target_os = "linux", feature = "batched-udp"))]
pub use linux::{recv_some, send_all};
#[cfg(all(target_os = "linux", feature = "batched-udp"))]
mod linux {
    use super::*;
    use std::io::{IoSlice, IoSliceMut};
    use std::mem::MaybeUninit;
    use realm_io::mmsg::{MmsgHdr, MmsgHdrMut};
    use realm_io::mmsg::{send_mul_pkts, recv_mul_pkts};

    pub async fn recv_some(sock: &UdpSocket, pkts: &mut [Packet]) -> Result<usize> {
        const MAX_PKTS: usize = MAX_PACKETS;
        debug_assert!(pkts.len() <= MAX_PKTS);

        let pkt_amt = pkts.len();
        let mut iovs: MaybeUninit<[IoSliceMut; MAX_PKTS]> = MaybeUninit::uninit();
        let mut msgs: MaybeUninit<[MmsgHdrMut; MAX_PKTS]> = MaybeUninit::uninit();
        let iovs = unsafe { iovs.assume_init_mut() };
        let msgs = unsafe { msgs.assume_init_mut() };

        for ((pkt, iov), msg) in pkts.iter_mut().zip(iovs.iter_mut()).zip(msgs.iter_mut()) {
            *iov = IoSliceMut::new(&mut pkt.buf);
            *msg = MmsgHdrMut::new()
                .with_addr(&mut pkt.addr.inner)
                .with_iovec(std::slice::from_mut(iov))
        }

        let pkt_amt = recv_mul_pkts(sock, &mut msgs[..pkt_amt]).await?;
        {
            let mut bytes: [u16; MAX_PKTS] = unsafe { std::mem::zeroed() };
            for (msg, byte) in msgs.iter().zip(bytes.iter_mut()).take(pkt_amt) {
                *byte = msg.get_ref().nbytes() as u16
            }

            for (pkt, byte) in pkts.iter_mut().zip(bytes).take(pkt_amt) {
                pkt.cursor = byte
            }
        }
        Ok(pkt_amt)
    }

    pub async fn send_all<'a, 'b, I>(sock: &UdpSocket, pkts: I) -> Result<()>
    where
        I: ExactSizeIterator<Item = PacketRef<'a, 'b>>,
    {
        const MAX_PKTS: usize = MAX_PACKETS;
        debug_assert!(pkts.len() <= MAX_PKTS);

        let pkt_amt = pkts.len();
        let mut iovs: MaybeUninit<[IoSlice; MAX_PKTS]> = MaybeUninit::uninit();
        let mut msgs: MaybeUninit<[MmsgHdr; MAX_PKTS]> = MaybeUninit::uninit();
        let iovs = unsafe { iovs.assume_init_mut() };
        let msgs = unsafe { msgs.assume_init_mut() };

        for ((pkt, iov), msg) in pkts.zip(iovs.iter_mut()).zip(msgs.iter_mut()) {
            *iov = IoSlice::new(pkt.buf);
            *msg = MmsgHdr::new()
                .with_addr(&pkt.addr.inner)
                .with_iovec(std::slice::from_ref(iov))
        }

        let mut cursor = 0;
        while cursor < pkt_amt {
            let n = send_mul_pkts(sock, &mut msgs[cursor..pkt_amt]).await?;
            cursor += n;
        }
        Ok(())
    }
}
