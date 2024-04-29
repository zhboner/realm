//! Mmsg impl.

use std::task::{Poll, Context};
use std::io::Result;
use std::os::unix::io::RawFd;

use crate::AsyncRawIO;

pub use store::{PacketStore, Const, Mutable};
pub use store::{PacketRef, PacketMutRef};
pub use store::{SockAddrStore, SOCK_STORE_LEN};
pub type Packet<'a, 'buf, 'iov, 'ctrl> = PacketStore<'a, 'buf, 'iov, 'ctrl, Const>;
pub type PacketMut<'a, 'buf, 'iov, 'ctrl> = PacketStore<'a, 'buf, 'iov, 'ctrl, Mutable>;

#[inline]
fn sendmpkts<M>(fd: RawFd, pkts: &mut [PacketStore<'_, '_, '_, '_, M>]) -> i32 {
    unsafe { libc::sendmmsg(fd, pkts.as_ptr() as *mut _, pkts.len() as u32, 0) }
}

#[inline]
fn recvmpkts(fd: RawFd, pkts: &mut [PacketMut]) -> i32 {
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

fn poll_sendmpkts<S, M>(
    stream: &mut S,
    cx: &mut Context<'_>,
    pkts: &mut [PacketStore<'_, '_, '_, '_, M>],
) -> Poll<Result<usize>>
where
    S: AsyncRawIO + Unpin,
{
    stream.poll_write_raw(cx, || sendmpkts(stream.as_raw_fd(), pkts) as isize)
}

fn poll_recvmpkts<S>(stream: &mut S, cx: &mut Context<'_>, pkts: &mut [PacketMut]) -> Poll<Result<usize>>
where
    S: AsyncRawIO + Unpin,
{
    stream.poll_read_raw(cx, || recvmpkts(stream.as_raw_fd(), pkts) as isize)
}

/// Send multiple packets.
pub async fn send_mul_pkts<S>(stream: &mut S, pkts: &mut [Packet<'_, '_, '_, '_>]) -> Result<usize>
where
    S: AsyncRawIO + Unpin,
{
    std::future::poll_fn(move |cx| poll_sendmpkts(stream, cx, pkts)).await
}

/// Recv multiple packets.
pub async fn recv_mul_pkts<S>(stream: &mut S, pkts: &mut [PacketMut<'_, '_, '_, '_>]) -> Result<usize>
where
    S: AsyncRawIO + Unpin,
{
    std::future::poll_fn(move |cx| poll_recvmpkts(stream, cx, pkts)).await
}

mod store {
    use std::{mem, ptr, slice};
    use std::marker::PhantomData;
    use std::io::{IoSlice, IoSliceMut};
    use std::net::SocketAddr;
    use socket2::SockAddr;
    use libc::{msghdr, mmsghdr};
    use libc::{sockaddr_storage, socklen_t};

    /// Marker.
    pub struct Const {}

    /// Marker.
    pub struct Mutable {}

    /// Represent [`libc::mmsghdr`].
    #[derive(Clone, Copy)]
    #[repr(transparent)]
    pub struct PacketStore<'a, 'buf, 'iov, 'ctrl, M> {
        pub(crate) store: mmsghdr,
        _type: PhantomData<M>,
        _lifetime: PhantomData<(&'a (), &'buf (), &'iov (), &'ctrl ())>,
    }

    /// Constant field accessor for [`PacketStore`].
    pub struct PacketRef<'a, 'buf, 'iov, 'ctrl, 'this> {
        addr: &'a SockAddrStore,
        iovec: &'iov [IoSlice<'buf>],
        control: &'ctrl [u8],
        flags: i32,
        nbytes: u32,
        _lifetime: PhantomData<&'this ()>,
    }

    /// Mutable field accessor for [`PacketStore`].
    pub struct PacketMutRef<'a, 'buf, 'iov, 'ctrl, 'this> {
        addr: &'a mut SockAddrStore,
        iovec: &'iov mut [IoSlice<'buf>],
        control: &'ctrl mut [u8],
        flags: i32,
        nbytes: u32,
        _lifetime: PhantomData<&'this ()>,
    }

    #[rustfmt::skip]
    macro_rules! access_fn {
        (!ref, $field: ident, $type: ty) => {
            pub fn $field(&self) -> $type { &self.$field }
        };
        (!mut, $field: ident, $type: ty) => {
            pub fn $field(&mut self) -> $type { &mut self.$field }
        };
        (!val, $field: ident, $type: ty) => {
            pub fn $field(&self) -> $type { self.$field }
        };
    }

    impl<'a, 'buf, 'iov, 'ctrl, 'this> PacketRef<'a, 'buf, 'iov, 'ctrl, 'this> {
        access_fn!(!ref, addr, &&'a SockAddrStore);
        access_fn!(!ref, iovec, &&'iov [IoSlice<'buf>]);
        access_fn!(!ref, control, &&'ctrl [u8]);
        access_fn!(!val, flags, i32);
        access_fn!(!val, nbytes, u32);
    }

    impl<'a, 'buf, 'iov, 'ctrl, 'this> PacketMutRef<'a, 'buf, 'iov, 'ctrl, 'this> {
        access_fn!(!mut, addr, &mut &'a mut SockAddrStore);
        access_fn!(!mut, iovec, &mut &'iov mut [IoSlice<'buf>]);
        access_fn!(!mut, control, &mut &'ctrl mut [u8]);
        access_fn!(!val, flags, i32);
        access_fn!(!val, nbytes, u32);
    }

    impl<'a, 'buf, 'iov, 'ctrl, M> PacketStore<'a, 'buf, 'iov, 'ctrl, M> {
        /// New zeroed storage.
        pub const fn new() -> Self {
            Self {
                store: unsafe { mem::zeroed::<mmsghdr>() },
                _type: PhantomData,
                _lifetime: PhantomData,
            }
        }

        /// Get constant accessor.
        #[rustfmt::skip]
        pub fn get_ref<'this>(&'this self) -> PacketRef<'this, 'a, 'buf, 'iov, 'ctrl> {
            let msghdr {
                msg_name, msg_namelen,
                msg_iov, msg_iovlen,
                msg_control, msg_controllen, msg_flags,
            } = self.store.msg_hdr;
            let msg_len = self.store.msg_len;
            unsafe { PacketRef {
                addr: &*msg_name.cast(), // todo!
                iovec: slice::from_raw_parts(msg_iov as *const _, msg_iovlen),
                control: slice::from_raw_parts(msg_control as *const _, msg_controllen),
                flags: msg_flags,
                nbytes: msg_len,
                _lifetime: PhantomData,
            }}
        }
    }

    impl<'a, 'buf, 'iov, 'ctrl, M> Default for PacketStore<'a, 'buf, 'iov, 'ctrl, M> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl<'a, 'buf, 'iov, 'ctrl> PacketStore<'a, 'buf, 'iov, 'ctrl, Const> {
        /// Set target address.
        pub const fn with_addr(mut self, addr: &'a SockAddrStore) -> Self {
            self.store.msg_hdr.msg_name = addr.0.as_ptr() as *mut _;
            self.store.msg_hdr.msg_namelen = addr.0.len();
            self
        }

        /// Set data to send.
        pub const fn with_iovec(mut self, iov: &'iov [IoSlice<'buf>]) -> Self {
            self.store.msg_hdr.msg_iov = ptr::from_ref(iov) as *mut _;
            self.store.msg_hdr.msg_iovlen = iov.len();
            self
        }

        /// Set control message to send.
        pub const fn with_control(mut self, ctrl: &'ctrl [u8]) -> Self {
            self.store.msg_hdr.msg_control = ptr::from_ref(ctrl) as *mut _;
            self.store.msg_hdr.msg_controllen = ctrl.len();
            self
        }

        /// Set sending flags.
        pub const fn with_flags(mut self, flags: i32) -> Self {
            self.store.msg_hdr.msg_flags = flags;
            self
        }
    }

    impl<'a, 'buf, 'iov, 'ctrl> PacketStore<'a, 'buf, 'iov, 'ctrl, Mutable> {
        /// Set storage to accommodate peer address.
        pub fn with_addr(mut self, addr: &'a mut SockAddrStore) -> Self {
            self.store.msg_hdr.msg_name = addr.0.as_ptr() as *mut _;
            self.store.msg_hdr.msg_namelen = addr.0.len();
            self
        }

        /// Set storage to receive data.
        pub fn with_iovec(mut self, iov: &'iov mut [IoSliceMut<'buf>]) -> Self {
            self.store.msg_hdr.msg_iov = ptr::from_mut(iov) as *mut _;
            self.store.msg_hdr.msg_iovlen = iov.len();
            self
        }

        /// Set storage to receive control message.
        pub fn with_control(mut self, ctrl: &'ctrl mut [u8]) -> Self {
            self.store.msg_hdr.msg_control = ptr::from_mut(ctrl) as *mut _;
            self.store.msg_hdr.msg_controllen = ctrl.len();
            self
        }

        /// Get mutable accessor.
        #[rustfmt::skip]
        pub fn get_mut<'this>(&'this mut self) -> PacketMutRef<'this, 'a, 'buf, 'iov, 'ctrl> {
            let msghdr {
                msg_name, msg_namelen,
                msg_iov, msg_iovlen,
                msg_control, msg_controllen, msg_flags,
            } = self.store.msg_hdr;
            let msg_len = self.store.msg_len;
            unsafe { PacketMutRef {
                addr: &mut *msg_name.cast(), // todo!
                iovec: slice::from_raw_parts_mut(msg_iov as *mut _, msg_iovlen),
                control: slice::from_raw_parts_mut(msg_control as *mut _, msg_controllen),
                flags: msg_flags,
                nbytes: msg_len,
                _lifetime: PhantomData,
            }}
        }
    }

    /// Represent [`libc::sockaddr_storage`].
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SockAddrStore(pub(crate) SockAddr);

    /// Size of [`libc::sockaddr_storage`].
    pub const SOCK_STORE_LEN: socklen_t = mem::size_of::<sockaddr_storage>() as socklen_t;

    impl SockAddrStore {
        /// New zeroed storage.
        pub const fn new() -> Self {
            Self(unsafe { SockAddr::new(mem::zeroed::<sockaddr_storage>(), SOCK_STORE_LEN) })
        }
    }

    impl Default for SockAddrStore {
        fn default() -> Self {
            Self::new()
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
