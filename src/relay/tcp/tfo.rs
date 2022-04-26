use std::io::Result;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Poll, Context};

use tokio_tfo::{TfoStream, TfoListener};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::io::ReadBuf;
use tokio::io::Interest;
use tokio::net::TcpSocket;

pub struct TcpStream(TfoStream);
pub struct TcpListener(TfoListener);

#[allow(clippy::mut_from_ref)]
#[inline]
unsafe fn const_cast<T>(x: &T) -> &mut T {
    let const_ptr = x as *const T;
    let mut_ptr = const_ptr as *mut T;
    &mut *mut_ptr
}

macro_rules! inner {
    ($x :ident) => {
        unsafe { const_cast(&$x.0).inner() }
    };
}

impl TcpListener {
    pub async fn bind(addr: SocketAddr) -> Result<TcpListener> {
        TfoListener::bind(addr).await.map(TcpListener)
    }

    pub async fn accept(&self) -> Result<(TcpStream, SocketAddr)> {
        self.0.accept().await.map(|(x, addr)| (TcpStream(x), addr))
    }
}

impl TcpStream {
    pub async fn connect_with_socket(socket: TcpSocket, addr: SocketAddr) -> Result<TcpStream> {
        TfoStream::connect_with_socket(socket, addr).await.map(TcpStream)
    }

    pub fn peer_addr(&self) -> Result<SocketAddr> {
        self.0.peer_addr()
    }

    pub fn set_nodelay(&self, nodelay: bool) -> Result<()> {
        self.0.set_nodelay(nodelay)
    }

    pub fn poll_read_ready(&self, cx: &mut Context<'_>) -> Poll<Result<()>> {
        inner!(self).poll_read_ready(cx)
    }

    pub fn poll_write_ready(&self, cx: &mut Context<'_>) -> Poll<Result<()>> {
        inner!(self).poll_write_ready(cx)
    }

    #[allow(unused)]
    pub fn try_io<R>(&self, interest: Interest, f: impl FnOnce() -> Result<R>) -> Result<R> {
        inner!(self).try_io(interest, f)
    }

    #[allow(unused)]
    pub async fn peek(&self, buf: &mut [u8]) -> Result<usize> {
        inner!(self).peek(buf).await
    }
}

impl From<TfoStream> for TcpStream {
    fn from(x: TfoStream) -> Self {
        TcpStream(x)
    }
}

impl From<tokio::net::TcpStream> for TcpStream {
    fn from(x: tokio::net::TcpStream) -> Self {
        TcpStream(x.into())
    }
}

impl AsyncRead for TcpStream {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.get_mut().0).poll_read(cx, buf)
    }
}

impl AsyncWrite for TcpStream {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        Pin::new(&mut self.get_mut().0).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.get_mut().0).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.get_mut().0).poll_shutdown(cx)
    }
}

#[cfg(target_os = "linux")]
mod linux_ext {
    use super::*;
    use std::os::unix::io::AsRawFd;
    use std::os::unix::prelude::RawFd;
    impl AsRawFd for TcpStream {
        fn as_raw_fd(&self) -> RawFd {
            self.0.as_raw_fd()
        }
    }

    use realm_io::AsyncRawIO;
    use realm_io::delegate_impl;

    delegate_impl!(TcpStream);
}
