use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "tfo")] {
        use tfo::TcpStream;
        use tfo::{ReadHalf, WriteHalf};
        pub use tfo::TcpListener;
    } else {
        use tokio::net::TcpStream;
        use tokio::net::tcp::{ReadHalf,WriteHalf};
        pub use tokio::net::TcpListener;
    }
}

cfg_if! {
    if #[cfg(all(target_os = "linux", feature = "zero-copy"))] {
        const BUFFER_SIZE: usize = 0x10000;
    } else {
        const BUFFER_SIZE: usize = 0x4000;
    }
}

use std::io::Result;
use std::fmt::{Display, Formatter};
use std::net::SocketAddr;
use std::time::Duration;
use futures::try_join;

use log::{debug, info};

use tokio::net::TcpSocket;
use tokio::time::timeout as timeoutfut;

use crate::utils::{RemoteAddr, ConnectOpts};

#[derive(Clone, Copy)]
pub enum TcpDirection {
    Forward,
    Reverse,
}

impl Display for TcpDirection {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use TcpDirection::*;
        match self {
            Forward => write!(f, "forward"),
            Reverse => write!(f, "reverse"),
        }
    }
}

#[allow(unused_variables)]
pub async fn proxy(
    mut inbound: TcpStream,
    remote: RemoteAddr,
    conn_opts: ConnectOpts,
) -> Result<()> {
    let ConnectOpts {
        tcp_timeout: timeout,
        fast_open,
        zero_copy,
        send_through,
        ..
    } = conn_opts;

    let remote = remote.into_sockaddr().await?;
    let timeout = Duration::from_secs(timeout as u64);

    let mut outbound = match send_through {
        Some(x) => {
            let socket = match x {
                SocketAddr::V4(_) => TcpSocket::new_v4()?,
                SocketAddr::V6(_) => TcpSocket::new_v6()?,
            };
            socket.set_reuseaddr(true)?;
            #[cfg(unix)]
            socket.set_reuseport(true)?;
            socket.bind(x)?;

            #[cfg(feature = "tfo")]
            if fast_open {
                TcpStream::connect_with_socket(socket, remote).await?
            } else {
                socket.connect(remote).await?.into()
            }

            #[cfg(not(feature = "tfo"))]
            socket.connect(remote).await?
        }
        None => TcpStream::connect(remote).await?,
    };

    info!("new tcp connection to remote {}", &remote);

    inbound.set_nodelay(true)?;
    outbound.set_nodelay(true)?;

    let (ri, wi) = inbound.split();
    let (ro, wo) = outbound.split();

    use TcpDirection::{Forward, Reverse};

    #[cfg(all(target_os = "linux", feature = "zero-copy"))]
    let res = if zero_copy {
        use zero_copy::copy;
        try_join!(
            copy(ri, wo, timeout, Forward),
            copy(ro, wi, timeout, Reverse)
        )
    } else {
        use normal_copy::copy;
        try_join!(
            copy(ri, wo, timeout, Forward),
            copy(ro, wi, timeout, Reverse)
        )
    };

    #[cfg(not(all(target_os = "linux", feature = "zero-copy")))]
    let res = {
        use normal_copy::copy;
        try_join!(
            copy(ri, wo, timeout, Forward),
            copy(ro, wi, timeout, Reverse)
        )
    };

    info!("tcp forward compelete or abort, close these 2 connection");

    // ignore read/write n bytes
    res.map(|_| ())
}

mod normal_copy {
    use super::*;

    #[allow(unused)]
    pub async fn copy(
        mut r: ReadHalf<'_>,
        mut w: WriteHalf<'_>,
        timeout: Duration,
        direction: TcpDirection,
    ) -> Result<()>
where {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let mut buf = vec![0u8; BUFFER_SIZE];
        let mut n: usize;
        loop {
            n = timeoutfut(timeout, r.read(&mut buf)).await??;
            if n == 0 {
                break;
            }
            w.write_all(&buf[..n]).await?;
            w.flush().await?;
        }
        w.shutdown().await?;

        debug!("tcp forward half-complete, direction: {}", direction);

        Ok(())
    }
}

#[cfg(all(target_os = "linux", feature = "zero-copy"))]
mod zero_copy {
    use super::*;
    use std::ops::Drop;
    use std::io::{Error, ErrorKind};
    use tokio::io::Interest;

    struct Pipe(pub i32, pub i32);

    impl Drop for Pipe {
        fn drop(&mut self) {
            unsafe {
                libc::close(self.0);
                libc::close(self.1);
            }
        }
    }

    impl Pipe {
        fn create() -> Result<Self> {
            use libc::{c_int, O_NONBLOCK};
            let mut pipes = std::mem::MaybeUninit::<[c_int; 2]>::uninit();
            unsafe {
                if libc::pipe2(pipes.as_mut_ptr() as *mut c_int, O_NONBLOCK) < 0
                {
                    return Err(Error::new(
                        ErrorKind::Other,
                        "failed to create a pipe",
                    ));
                }
                Ok(Pipe(pipes.assume_init()[0], pipes.assume_init()[1]))
            }
        }
    }

    #[inline]
    fn splice_n(r: i32, w: i32, n: usize) -> isize {
        use libc::{loff_t, SPLICE_F_MOVE, SPLICE_F_NONBLOCK};
        unsafe {
            libc::splice(
                r,
                std::ptr::null_mut::<loff_t>(),
                w,
                std::ptr::null_mut::<loff_t>(),
                n,
                SPLICE_F_MOVE | SPLICE_F_NONBLOCK,
            )
        }
    }

    #[inline]
    fn is_wouldblock() -> bool {
        use libc::{EWOULDBLOCK, EAGAIN};
        let err = Error::last_os_error().raw_os_error();
        match err {
            Some(e) => e == EWOULDBLOCK || e == EAGAIN,
            None => false,
        }
    }

    #[inline]
    fn clear_readiness(x: &TcpStream, interest: Interest) {
        let _ = x.try_io(interest, || {
            Err(Error::new(ErrorKind::WouldBlock, "")) as Result<()>
        });
    }

    pub async fn copy(
        r: ReadHalf<'_>,
        mut w: WriteHalf<'_>,
        timeout: Duration,
        direction: TcpDirection,
    ) -> Result<()> {
        use std::os::unix::io::AsRawFd;
        use tokio::io::AsyncWriteExt;
        // init pipe
        let pipe = Pipe::create()?;
        let (rpipe, wpipe) = (pipe.0, pipe.1);
        // rw ref
        let rx = r.as_ref();
        let wx = w.as_ref();
        // rw raw fd
        let rfd = rx.as_raw_fd();
        let wfd = wx.as_raw_fd();
        // ctrl
        let mut n: usize = 0;
        let mut done = false;

        let res = 'LOOP: loop {
            // read until the socket buffer is empty
            // or the pipe is filled
            timeoutfut(timeout, rx.readable()).await??;
            while n < BUFFER_SIZE {
                match splice_n(rfd, wpipe, BUFFER_SIZE - n) {
                    x if x > 0 => n += x as usize,
                    x if x == 0 => {
                        done = true;
                        break;
                    }
                    x if x < 0 && is_wouldblock() => {
                        clear_readiness(rx, Interest::READABLE);
                        break;
                    }
                    _ => {
                        break 'LOOP Err(Error::new(
                            ErrorKind::Other,
                            "failed to splice from tcp connection",
                        ))
                    }
                }
            }
            // write until the pipe is empty
            while n > 0 {
                wx.writable().await?;
                match splice_n(rpipe, wfd, n) {
                    x if x > 0 => n -= x as usize,
                    x if x < 0 && is_wouldblock() => {
                        clear_readiness(wx, Interest::WRITABLE);
                    }
                    _ => {
                        break 'LOOP Err(Error::new(
                            ErrorKind::Other,
                            "failed to splice to tcp connection",
                        ))
                    }
                }
            }
            // complete
            if done {
                break Ok(());
            }
        };

        if done {
            w.shutdown().await?;
            debug!("tcp forward half-complete, direction: {}", direction);
        };
        res
    }
}

#[cfg(feature = "tfo")]
mod tfo {
    use std::io::Result;
    use std::net::SocketAddr;
    use std::pin::Pin;
    use std::task::{Poll, Context};

    use tokio_tfo::{TfoStream, TfoListener};
    use tokio::io::{AsyncRead, AsyncWrite};
    use tokio::io::ReadBuf;
    use tokio::io::Interest;
    use tokio::net::TcpSocket;

    pub struct TcpListener(TfoListener);

    pub struct TcpStream(TfoStream);
    pub struct ReadHalf<'a>(&'a TcpStream);
    pub struct WriteHalf<'a>(&'a TcpStream);

    #[allow(clippy::mut_from_ref)]
    #[inline]
    unsafe fn const_cast<T>(x: &T) -> &mut T {
        let const_ptr = x as *const T;
        let mut_ptr = const_ptr as *mut T;
        &mut *mut_ptr
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
        pub async fn connect(addr: SocketAddr) -> Result<TcpStream> {
            TfoStream::connect(addr).await.map(TcpStream)
        }

        pub async fn connect_with_socket(
            socket: TcpSocket,
            addr: SocketAddr,
        ) -> Result<TcpStream> {
            TfoStream::connect_with_socket(socket, addr)
                .await
                .map(TcpStream)
        }

        #[allow(unused)]
        pub async fn readable(&self) -> Result<()> {
            self.0.inner().readable().await
        }

        #[allow(unused)]
        pub async fn writable(&self) -> Result<()> {
            self.0.inner().writable().await
        }

        pub fn set_nodelay(&self, nodelay: bool) -> Result<()> {
            self.0.set_nodelay(nodelay)
        }

        pub fn split(&mut self) -> (ReadHalf, WriteHalf) {
            (ReadHalf(&*self), WriteHalf(&*self))
        }

        #[allow(unused)]
        pub fn try_io<R>(
            &self,
            interest: Interest,
            f: impl FnOnce() -> Result<R>,
        ) -> Result<R> {
            self.0.inner().try_io(interest, f)
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
        fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut ReadBuf<'_>,
        ) -> Poll<std::io::Result<()>> {
            Pin::new(&mut self.get_mut().0).poll_read(cx, buf)
        }
    }

    impl AsyncWrite for TcpStream {
        fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<Result<usize>> {
            Pin::new(&mut self.get_mut().0).poll_write(cx, buf)
        }

        fn poll_flush(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<()>> {
            Pin::new(&mut self.get_mut().0).poll_flush(cx)
        }

        fn poll_shutdown(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<()>> {
            Pin::new(&mut self.get_mut().0).poll_shutdown(cx)
        }
    }

    impl<'a> AsyncRead for ReadHalf<'a> {
        fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &mut ReadBuf<'_>,
        ) -> Poll<std::io::Result<()>> {
            Pin::new(unsafe { const_cast(self.0) }).poll_read(cx, buf)
        }
    }

    impl<'a> AsyncWrite for WriteHalf<'a> {
        fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<Result<usize>> {
            Pin::new(unsafe { const_cast(self.0) }).poll_write(cx, buf)
        }

        fn poll_flush(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<()>> {
            Pin::new(unsafe { const_cast(self.0) }).poll_flush(cx)
        }

        fn poll_shutdown(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
        ) -> Poll<Result<()>> {
            Pin::new(unsafe { const_cast(self.0) }).poll_shutdown(cx)
        }
    }

    impl<'a> AsRef<TcpStream> for ReadHalf<'a> {
        fn as_ref(&self) -> &TcpStream {
            self.0
        }
    }

    impl<'a> AsRef<TcpStream> for WriteHalf<'a> {
        fn as_ref(&self) -> &TcpStream {
            self.0
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
    }
}
