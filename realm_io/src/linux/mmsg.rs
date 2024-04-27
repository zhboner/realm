use std::task::{Poll, Context};
use std::io::Result;
use std::io::{IoSlice, IoSliceMut};
use std::os::unix::io::RawFd;

use crate::AsyncRawIO;

#[derive(Debug, Clone, Copy)]
pub struct Packet<'a, 'buf> {
    addr: &'a SockAddrStore,
    iovec: IoSlice<'buf>,
}

#[derive(Debug)]
pub struct PacketMut<'a, 'buf> {
    addr: &'a mut SockAddrStore,
    iovec: IoSliceMut<'buf>,
}

impl<'a, 'buf> Packet<'a, 'buf> {
    pub fn new(addr: &'a SockAddrStore, data: &'buf [u8]) -> Self {
        Self {
            addr,
            iovec: IoSlice::new(data),
        }
    }

    pub fn into_store<'pkt>(&'pkt self) -> PacketStore<'a, 'buf, 'pkt> {
        use std::marker::PhantomData;
        let Packet { addr, iovec } = self;
        PacketStore {
            msg: libc::msghdr {
                msg_name: addr.0.as_ptr() as *mut _,
                msg_namelen: addr.0.len(),
                msg_iov: iovec as *const IoSlice as *mut _,
                msg_iovlen: 1,
                msg_control: std::ptr::null_mut(),
                msg_controllen: 0,
                msg_flags: 0,
            },
            addr: PhantomData,
            iovec: PhantomData,
            packet: PhantomData,
        }
    }
}

impl<'a, 'buf> PacketMut<'a, 'buf> {
    pub fn new(addr: &'a mut SockAddrStore, data: &'buf mut [u8]) -> Self {
        Self {
            addr,
            iovec: IoSliceMut::new(data),
        }
    }

    pub fn into_store<'pkt>(&'pkt mut self) -> PacketStore<'a, 'buf, 'pkt> {
        use std::marker::PhantomData;
        let PacketMut { addr, iovec } = self;
        PacketStore {
            msg: libc::msghdr {
                msg_name: addr.0.as_ptr() as *mut _,
                msg_namelen: addr.0.len(),
                msg_iov: iovec as *mut IoSliceMut as *mut _,
                msg_iovlen: 1,
                msg_control: std::ptr::null_mut(),
                msg_controllen: 0,
                msg_flags: 0,
            },
            addr: PhantomData,
            iovec: PhantomData,
            packet: PhantomData,
        }
    }
}

#[inline]
fn sendmpkts(fd: RawFd, pkts: &[PacketStore]) -> i32 {
    unsafe { libc::sendmmsg(fd, pkts.as_ptr() as *mut _, pkts.len() as u32, 0) }
}

#[inline]
fn recvmpkts(fd: RawFd, pkts: &mut [PacketStore]) -> i32 {
    unsafe {
        libc::recvmmsg(
            fd,
            pkts.as_mut_ptr() as *mut _,
            pkts.len() as u32,
            0,
            std::ptr::null_mut(),
        )
    }
}

fn poll_sendmpkts<S>(stream: &mut S, cx: &mut Context<'_>, pkts: &[PacketStore]) -> Poll<Result<usize>>
where
    S: AsyncRawIO + Unpin,
{
    stream.poll_read_raw(cx, || sendmpkts(stream.as_raw_fd(), pkts) as isize)
}

fn poll_recvmpkts<S>(stream: &mut S, cx: &mut Context<'_>, pkts: &mut [PacketStore]) -> Poll<Result<usize>>
where
    S: AsyncRawIO + Unpin,
{
    stream.poll_write_raw(cx, || recvmpkts(stream.as_raw_fd(), pkts) as isize)
}

pub async fn send_mul_pkts<S>(stream: &mut S, pkts: &[PacketStore<'_, '_, '_>]) -> Result<usize>
where
    S: AsyncRawIO + Unpin,
{
    std::future::poll_fn(move |cx| poll_sendmpkts(stream, cx, pkts)).await
}

pub async fn recv_mul_pkts<S>(stream: &mut S, pkts: &mut [PacketStore<'_, '_, '_>]) -> Result<usize>
where
    S: AsyncRawIO + Unpin,
{
    std::future::poll_fn(move |cx| poll_recvmpkts(stream, cx, pkts)).await
}

pub use store::{PacketStore, SockAddrStore, STORE_LEN};
mod store {
    use std::mem;
    use std::marker::PhantomData;
    use std::net::SocketAddr;
    use socket2::SockAddr;
    use libc::{msghdr, sockaddr_storage, socklen_t};

    /// Represent [`libc::msghdr`].
    #[derive(Clone, Copy)]
    #[repr(transparent)]
    pub struct PacketStore<'a, 'buf, 'pkt> {
        pub(crate) msg: msghdr,
        pub(crate) addr: PhantomData<&'a ()>,
        pub(crate) iovec: PhantomData<&'buf ()>,
        pub(crate) packet: PhantomData<&'pkt ()>,
    }

    /// Represent [`libc::sockaddr_storage`].
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SockAddrStore(pub(crate) SockAddr);
    pub const STORE_LEN: socklen_t = mem::size_of::<sockaddr_storage>() as socklen_t;
    mod addr {
        use super::*;
        impl SockAddrStore {
            pub const fn new_zeroed() -> Self {
                Self(unsafe { SockAddr::new(mem::zeroed::<sockaddr_storage>(), STORE_LEN) })
            }
        }

        impl<T> From<T> for SockAddrStore
        where
            SockAddr: From<T>,
        {
            fn from(addr: T) -> Self {
                Self(addr.into())
            }
        }

        impl From<SockAddrStore> for SocketAddr {
            fn from(store: SockAddrStore) -> Self {
                store.0.as_socket().unwrap()
            }
        }
    }
}
