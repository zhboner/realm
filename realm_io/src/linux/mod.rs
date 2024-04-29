use std::io::{Result, Error, ErrorKind};
use std::task::{Poll, Context, ready};
use std::os::unix::io::AsRawFd;

use tokio::io::Interest;

pub mod mmsg;
pub mod zero_copy;

/// Type traits of Linux objects.
pub trait AsyncRawIO: AsRawFd {
    fn x_poll_read_ready(&self, cx: &mut Context<'_>) -> Poll<Result<()>>;
    fn x_poll_write_ready(&self, cx: &mut Context<'_>) -> Poll<Result<()>>;
    fn x_try_io<R>(&self, interest: Interest, f: impl FnOnce() -> Result<R>) -> Result<R>;

    fn check_wouldblock() -> Error {
        use libc::{EWOULDBLOCK, EAGAIN};
        let err = Error::last_os_error();
        match err.raw_os_error() {
            Some(e) if e == EWOULDBLOCK || e == EAGAIN => ErrorKind::WouldBlock.into(),
            _ => err,
        }
    }

    fn poll_read_raw<S>(&self, cx: &mut Context<'_>, mut syscall: S) -> Poll<Result<usize>>
    where
        S: FnMut() -> isize,
    {
        loop {
            ready!(Self::x_poll_read_ready(self, cx))?;
            match Self::x_try_io(self, Interest::READABLE, || match syscall() {
                x if x >= 0 => Ok(x as usize),
                _ => Err(Self::check_wouldblock()),
            }) {
                Ok(n) => return Poll::Ready(Ok(n)),
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => continue,
                Err(e) => return Poll::Ready(Err(e)),
            }
        }
    }

    fn poll_write_raw<S>(&self, cx: &mut Context<'_>, mut syscall: S) -> Poll<Result<usize>>
    where
        S: FnMut() -> isize,
    {
        loop {
            ready!(Self::x_poll_write_ready(self, cx))?;
            match Self::x_try_io(self, Interest::WRITABLE, || match syscall() {
                x if x >= 0 => Ok(x as usize),
                _ => Err(Self::check_wouldblock()),
            }) {
                Ok(n) => return Poll::Ready(Ok(n)),
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => continue,
                Err(e) => return Poll::Ready(Err(e)),
            }
        }
    }
}

mod tokio_net {
    use tokio::net::{TcpStream, UnixStream};
    use tokio::net::{UdpSocket, UnixDatagram};
    use super::AsyncRawIO;
    use super::*;

    trait IoReady {
        fn poll_read_ready(&self, cx: &mut Context<'_>) -> Poll<Result<()>>;
        fn poll_write_ready(&self, cx: &mut Context<'_>) -> Poll<Result<()>>;
    }
    macro_rules! workaround {
        ($socket: ident) => {
            impl IoReady for $socket {
                #[inline]
                fn poll_read_ready(&self, cx: &mut Context<'_>) -> Poll<Result<()>> {
                    self.poll_recv_ready(cx)
                }

                #[inline]
                fn poll_write_ready(&self, cx: &mut Context<'_>) -> Poll<Result<()>> {
                    self.poll_send_ready(cx)
                }
            }
        };
    }
    workaround!(UdpSocket);
    workaround!(UnixDatagram);

    /// Impl [`AsyncRawIO`], delegates to required functions.
    #[macro_export]
    macro_rules! delegate_impl {
        ($stream: ident) => {
            impl AsyncRawIO for $stream {
                #[inline]
                fn x_poll_read_ready(&self, cx: &mut Context<'_>) -> Poll<Result<()>> {
                    self.poll_read_ready(cx)
                }

                #[inline]
                fn x_poll_write_ready(&self, cx: &mut Context<'_>) -> Poll<Result<()>> {
                    self.poll_write_ready(cx)
                }

                #[inline]
                fn x_try_io<R>(&self, interest: Interest, f: impl FnOnce() -> Result<R>) -> Result<R> {
                    self.try_io(interest, f)
                }
            }
        };
    }

    delegate_impl!(TcpStream);
    delegate_impl!(UnixStream);
    delegate_impl!(UdpSocket);
    delegate_impl!(UnixDatagram);
}
