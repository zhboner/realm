//! Mmsg impl.

use std::task::{Poll, Context};
use std::io::Result;
use std::os::unix::io::RawFd;

use crate::AsyncRawIO;

pub use store::{MmsgHdrStore, Const, Mutable};
pub use store::{MmsgRef, MmsgMutRef};
pub use store::{SockAddrStore, SOCK_STORE_LEN};
pub type MmsgHdr<'a, 'b, 'iov, 'ctrl> = MmsgHdrStore<'a, 'b, 'iov, 'ctrl, Const>;
pub type MmsgHdrMut<'a, 'b, 'iov, 'ctrl> = MmsgHdrStore<'a, 'b, 'iov, 'ctrl, Mutable>;

#[inline]
fn sendmpkts<M>(fd: RawFd, pkts: &mut [MmsgHdrStore<'_, '_, '_, '_, M>]) -> i32 {
    unsafe { libc::sendmmsg(fd, pkts.as_ptr() as *mut _, pkts.len() as u32, 0) }
}

#[inline]
fn recvmpkts(fd: RawFd, pkts: &mut [MmsgHdrMut]) -> i32 {
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
    sock: &S,
    cx: &mut Context<'_>,
    pkts: &mut [MmsgHdrStore<'_, '_, '_, '_, M>],
) -> Poll<Result<usize>>
where
    S: AsyncRawIO + Unpin,
{
    sock.poll_write_raw(cx, || sendmpkts(sock.as_raw_fd(), pkts) as isize)
}

fn poll_recvmpkts<S>(sock: &S, cx: &mut Context<'_>, pkts: &mut [MmsgHdrMut]) -> Poll<Result<usize>>
where
    S: AsyncRawIO + Unpin,
{
    sock.poll_read_raw(cx, || recvmpkts(sock.as_raw_fd(), pkts) as isize)
}

/// Send multiple packets.
pub async fn send_mul_pkts<S, M>(sock: &S, pkts: &mut [MmsgHdrStore<'_, '_, '_, '_, M>]) -> Result<usize>
where
    S: AsyncRawIO + Unpin,
{
    std::future::poll_fn(move |cx| poll_sendmpkts(sock, cx, pkts)).await
}

/// Recv multiple packets.
pub async fn recv_mul_pkts<S>(sock: &S, pkts: &mut [MmsgHdrMut<'_, '_, '_, '_>]) -> Result<usize>
where
    S: AsyncRawIO + Unpin,
{
    std::future::poll_fn(move |cx| poll_recvmpkts(sock, cx, pkts)).await
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
    #[derive(Debug, Clone, Copy)]
    pub struct Const {}

    /// Marker.
    #[derive(Debug, Clone, Copy)]
    pub struct Mutable {}

    /// # Safety: mmsghdr is POD.
    unsafe impl<'a, 'b, 'iov, 'ctrl, M> Send for MmsgHdrStore<'a, 'b, 'iov, 'ctrl, M> {}

    /// # Safety: Inner pointers come from references ruled by the borrow-checker,
    /// thereby mutable pointers which point to the same memory address cant co-exist.
    ///
    /// We provide a thin and limited interface that neither [`Const`] nor &[`Mutable`]
    /// can have mutable access to the internal data, while restriction of mutable access
    /// behind `&mut`[`Mutable`] will be enforced by the borrow-checker.
    unsafe impl<'a, 'b, 'iov, 'ctrl, M> Sync for MmsgHdrStore<'a, 'b, 'iov, 'ctrl, M> {}

    /// Represent [`libc::mmsghdr`].
    #[repr(C)]
    pub struct MmsgHdrStore<'a, 'b, 'iov, 'ctrl, M> {
        pub(crate) store: mmsghdr,
        _type: PhantomData<M>,
        _lifetime: PhantomData<(&'a (), &'b (), &'iov (), &'ctrl ())>,
    }

    /// Constant field accessor for [`MmsgHdrStore`].
    pub struct MmsgRef<'a, 'b, 'iov, 'ctrl, 'this> {
        addr: &'a SockAddrStore,
        iovec: &'iov [IoSlice<'b>],
        control: &'ctrl [u8],
        flags: &'this i32,
        nbytes: u32,
        _lifetime: PhantomData<&'this ()>,
    }

    /// Mutable field accessor for [`MmsgHdrStore`].
    pub struct MmsgMutRef<'a, 'b, 'iov, 'ctrl, 'this> {
        addr: &'a mut SockAddrStore,
        iovec: &'iov mut [IoSlice<'b>],
        control: &'ctrl mut [u8],
        flags: &'this mut i32,
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

    impl<'a, 'b, 'iov, 'ctrl, 'this> MmsgRef<'a, 'b, 'iov, 'ctrl, 'this> {
        access_fn!(!ref, addr, &&'a SockAddrStore);
        access_fn!(!ref, iovec, &&'iov [IoSlice<'b>]);
        access_fn!(!ref, control, &&'ctrl [u8]);
        access_fn!(!ref, flags, &&'this i32);
        access_fn!(!val, nbytes, u32);
    }

    impl<'a, 'b, 'iov, 'ctrl, 'this> MmsgMutRef<'a, 'b, 'iov, 'ctrl, 'this> {
        access_fn!(!mut, addr, &mut &'a mut SockAddrStore);
        access_fn!(!mut, iovec, &mut &'iov mut [IoSlice<'b>]);
        access_fn!(!mut, control, &mut &'ctrl mut [u8]);
        access_fn!(!mut, flags, &mut &'this mut i32);
        access_fn!(!val, nbytes, u32);
    }

    #[inline]
    const unsafe fn make_slice<'a, T>(ptr: *const T, n: usize) -> &'a [T] {
        use std::ptr::NonNull;
        let ptr = if n != 0 { ptr } else { NonNull::dangling().as_ptr() };
        slice::from_raw_parts(ptr, n)
    }

    #[inline]
    unsafe fn make_slice_mut<'a, T>(ptr: *mut T, n: usize) -> &'a mut [T] {
        use std::ptr::NonNull;
        let ptr = if n != 0 { ptr } else { NonNull::dangling().as_ptr() };
        slice::from_raw_parts_mut(ptr, n)
    }

    impl<'a, 'b, 'iov, 'ctrl, M> MmsgHdrStore<'a, 'b, 'iov, 'ctrl, M> {
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
        pub fn get_ref<'this>(&'this self) -> MmsgRef<'a, 'b, 'iov, 'ctrl, 'this> {
            let msghdr {
                msg_name, msg_namelen: _,
                msg_iov, msg_iovlen,
                msg_control, msg_controllen, ..
            } = self.store.msg_hdr;
            unsafe { MmsgRef {
                addr: &*msg_name.cast(),
                iovec: make_slice(msg_iov as *const _, msg_iovlen as _),
                control: make_slice(msg_control as *const _, msg_controllen as _),
                flags: &self.store.msg_hdr.msg_flags,
                nbytes: self.store.msg_len,
                _lifetime: PhantomData,
            }}
        }
    }

    impl<'a, 'b, 'iov, 'ctrl, M> Default for MmsgHdrStore<'a, 'b, 'iov, 'ctrl, M> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl<'a, 'b, 'iov, 'ctrl> MmsgHdrStore<'a, 'b, 'iov, 'ctrl, Const> {
        /// Set target address.
        pub const fn with_addr(mut self, addr: &'a SockAddrStore) -> Self {
            self.store.msg_hdr.msg_name = addr.0.as_ptr() as *mut _;
            self.store.msg_hdr.msg_namelen = addr.0.len() as _;
            self
        }

        /// Set data to send.
        pub const fn with_iovec(mut self, iov: &'iov [IoSlice<'b>]) -> Self {
            self.store.msg_hdr.msg_iov = ptr::from_ref(iov) as *mut _;
            self.store.msg_hdr.msg_iovlen = iov.len() as _;
            self
        }

        /// Set control message to send.
        pub const fn with_control(mut self, ctrl: &'ctrl [u8]) -> Self {
            self.store.msg_hdr.msg_control = ptr::from_ref(ctrl) as *mut _;
            self.store.msg_hdr.msg_controllen = ctrl.len() as _;
            self
        }

        /// Set message flags to send.
        pub const fn with_flags(mut self, flags: i32) -> Self {
            self.store.msg_hdr.msg_flags = flags;
            self
        }
    }

    impl<'a, 'b, 'iov, 'ctrl> MmsgHdrStore<'a, 'b, 'iov, 'ctrl, Mutable> {
        /// Set storage to accommodate peer address.
        pub fn with_addr(mut self, addr: &'a mut SockAddrStore) -> Self {
            self.store.msg_hdr.msg_name = addr.0.as_ptr() as *mut _;
            self.store.msg_hdr.msg_namelen = addr.0.len() as _;
            self
        }

        /// Set storage to receive data.
        pub fn with_iovec(mut self, iov: &'iov mut [IoSliceMut<'b>]) -> Self {
            self.store.msg_hdr.msg_iov = ptr::from_mut(iov) as *mut _;
            self.store.msg_hdr.msg_iovlen = iov.len() as _;
            self
        }

        /// Set storage to receive control message.
        pub fn with_control(mut self, ctrl: &'ctrl mut [u8]) -> Self {
            self.store.msg_hdr.msg_control = ptr::from_mut(ctrl) as *mut _;
            self.store.msg_hdr.msg_controllen = ctrl.len() as _;
            self
        }

        /// Get mutable accessor.
        #[rustfmt::skip]
        pub fn get_mut<'this>(&'this mut self) -> MmsgMutRef<'a, 'b, 'iov, 'ctrl, 'this> {
            let msghdr {
                msg_name, msg_namelen: _,
                msg_iov, msg_iovlen,
                msg_control, msg_controllen, ..
            } = self.store.msg_hdr;
            unsafe { MmsgMutRef {
                addr: &mut *msg_name.cast(),
                iovec: make_slice_mut(msg_iov as *mut _, msg_iovlen as _),
                control: make_slice_mut(msg_control as *mut _, msg_controllen as _),
                flags: &mut self.store.msg_hdr.msg_flags,
                nbytes: self.store.msg_len,
                _lifetime: PhantomData,
            }}
        }
    }

    /// Represent [`libc::sockaddr_storage`].
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    #[repr(C)]
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
