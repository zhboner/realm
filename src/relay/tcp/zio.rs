use std::io::{Result, Error, ErrorKind};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::future::Future;

use futures::ready;

use super::TcpStream;
use tokio::io::ReadBuf;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::utils::DEFAULT_BUF_SIZE;
trait Require
where
    Self: Sized,
{
    fn new() -> Result<Self>;

    fn poll_read_tcp(
        &mut self,
        cx: &mut Context<'_>,
        stream: &mut TcpStream,
    ) -> Poll<Result<usize>>;

    fn poll_write_tcp(
        &mut self,
        cx: &mut Context<'_>,
        stream: &mut TcpStream,
    ) -> Poll<Result<usize>>;

    fn poll_flush_tcp(
        &mut self,
        cx: &mut Context<'_>,
        stream: &mut TcpStream,
    ) -> Poll<Result<()>>;
}

struct CopyBuffer<B = Box<[u8]>> {
    read_done: bool,
    need_flush: bool,
    pos: usize,
    cap: usize,
    amt: u64,
    buf: B,
}

// use array buffer by default
impl Require for CopyBuffer {
    fn new() -> Result<Self> {
        Ok(Self {
            read_done: false,
            need_flush: false,
            pos: 0,
            cap: 0,
            amt: 0,
            buf: vec![0; DEFAULT_BUF_SIZE].into_boxed_slice(),
        })
    }

    fn poll_read_tcp(
        &mut self,
        cx: &mut Context<'_>,
        stream: &mut TcpStream,
    ) -> Poll<Result<usize>> {
        let mut buf = ReadBuf::new(&mut self.buf);
        Pin::new(stream)
            .poll_read(cx, &mut buf)
            .map_ok(|_| buf.filled().len())
    }

    fn poll_write_tcp(
        &mut self,
        cx: &mut Context<'_>,
        stream: &mut TcpStream,
    ) -> Poll<Result<usize>> {
        Pin::new(stream).poll_write(cx, &self.buf[self.pos..self.cap])
    }

    fn poll_flush_tcp(
        &mut self,
        cx: &mut Context<'_>,
        stream: &mut TcpStream,
    ) -> Poll<Result<()>> {
        Pin::new(stream).poll_flush(cx)
    }
}

impl<B> CopyBuffer<B>
where
    CopyBuffer<B>: Require,
{
    fn poll_copy(
        &mut self,
        cx: &mut Context<'_>,
        r: &mut TcpStream,
        w: &mut TcpStream,
    ) -> Poll<Result<()>> {
        loop {
            // If our buffer is empty, then we need to read some data to
            // continue.
            if self.pos == self.cap && !self.read_done {
                let n = match self.poll_read_tcp(cx, r) {
                    Poll::Ready(Ok(n)) => n,
                    Poll::Ready(Err(err)) => return Poll::Ready(Err(err)),
                    Poll::Pending => {
                        // Try flushing when the reader has no progress to avoid deadlock
                        // when the reader depends on buffered writer.
                        if self.need_flush {
                            ready!(self.poll_flush_tcp(cx, w))?;
                            self.need_flush = false;
                        }

                        return Poll::Pending;
                    }
                };

                if n == 0 {
                    self.read_done = true;
                } else {
                    self.pos = 0;
                    self.cap = n;
                }
            }

            // If our buffer has some data, let's write it out!
            // Note: send may return ECONNRESET but splice wont, see
            // https://man7.org/linux/man-pages/man2/send.2.html
            // https://man7.org/linux/man-pages/man2/splice.2.html
            while self.pos < self.cap {
                let i = ready!(self.poll_write_tcp(cx, w))?;

                if i == 0 {
                    return Poll::Ready(Err(Error::new(
                        ErrorKind::WriteZero,
                        "write zero byte into writer",
                    )));
                } else {
                    self.pos += i;
                    self.amt += i as u64;
                    self.need_flush = true;
                }
            }

            // If pos larger than cap, this loop will never stop.
            // In particular, user's wrong poll_write implementation returning
            // incorrect written length may lead to thread blocking.
            debug_assert!(
                self.pos <= self.cap,
                "writer returned length larger than input slice"
            );

            // If we've written all the data and we've seen EOF, flush out the
            // data and finish the transfer.
            if self.pos == self.cap && self.read_done {
                ready!(self.poll_flush_tcp(cx, w))?;
                return Poll::Ready(Ok(()));
            }
        }
    }
}

enum TransferState<B> {
    Running(CopyBuffer<B>),
    ShuttingDown,
    Done,
}

struct BidiCopy<'a, B> {
    a: &'a mut TcpStream,
    b: &'a mut TcpStream,
    a_to_b: TransferState<B>,
    b_to_a: TransferState<B>,
}

fn transfer_one_direction<B>(
    cx: &mut Context<'_>,
    state: &mut TransferState<B>,
    r: &mut TcpStream,
    w: &mut TcpStream,
) -> Poll<Result<()>>
where
    CopyBuffer<B>: Require,
{
    loop {
        match state {
            TransferState::Running(buf) => {
                ready!(buf.poll_copy(cx, r, w))?;

                *state = TransferState::ShuttingDown;
            }
            TransferState::ShuttingDown => {
                ready!(Pin::new(&mut *w).poll_shutdown(cx))?;

                *state = TransferState::Done;
            }
            TransferState::Done => return Poll::Ready(Ok(())),
        }
    }
}

impl<'a, B> Future for BidiCopy<'a, B>
where
    B: Unpin,
    CopyBuffer<B>: Require,
{
    type Output = Result<()>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        // Unpack self into mut refs to each field to avoid borrow check issues.
        let BidiCopy {
            a,
            b,
            a_to_b,
            b_to_a,
        } = &mut *self;

        let a_to_b = transfer_one_direction(cx, a_to_b, a, b)?;
        let b_to_a = transfer_one_direction(cx, b_to_a, b, a)?;

        // It is not a problem if ready! returns early because transfer_one_direction for the
        // other direction will keep returning TransferState::Done(count) in future calls to poll
        ready!(a_to_b);
        ready!(b_to_a);

        Poll::Ready(Ok(()))
    }
}

// async fn bidi_copy<B>(
//     a: &mut TcpStream,
//     b: &mut TcpStream,
// ) -> Result<(())>
// where
//     B: Unpin,
//     CopyBuffer<B>: Require,
// {
//     let a_to_b = TransferState::Running(CopyBuffer::<B>::new()?);
//     let b_to_a = TransferState::Running(CopyBuffer::<B>::new()?);
//     BidiCopy {
//         a,
//         b,
//         a_to_b,
//         b_to_a,
//     }
//     .await
// }

pub async fn bidi_copy_buffer(
    a: &mut TcpStream,
    b: &mut TcpStream,
) -> Result<()> {
    let a_to_b =
        TransferState::Running(CopyBuffer::<Box<[u8]>>::new().unwrap());
    let b_to_a =
        TransferState::Running(CopyBuffer::<Box<[u8]>>::new().unwrap());
    BidiCopy {
        a,
        b,
        a_to_b,
        b_to_a,
    }
    .await
}

#[cfg(all(target_os = "linux", feature = "zero-copy"))]
pub async fn bidi_copy_pipe(
    a: &mut TcpStream,
    b: &mut TcpStream,
) -> Result<()> {
    use zero_copy::Pipe;
    let a_to_b = TransferState::Running(CopyBuffer::<Pipe>::new()?);
    let b_to_a = TransferState::Running(CopyBuffer::<Pipe>::new()?);
    BidiCopy {
        a,
        b,
        a_to_b,
        b_to_a,
    }
    .await
}

#[cfg(all(target_os = "linux", feature = "zero-copy"))]
mod zero_copy {
    use super::*;
    use std::ops::Drop;
    use std::os::unix::io::{RawFd, AsRawFd};
    use tokio::io::Interest;
    use crate::utils::DEFAULT_PIPE_CAP;
    use crate::utils::CUSTOM_PIPE_CAP;

    pub struct Pipe(RawFd, RawFd);

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
            let mut pipe = std::mem::MaybeUninit::<[c_int; 2]>::uninit();
            unsafe {
                if libc::pipe2(pipe.as_mut_ptr() as *mut c_int, O_NONBLOCK) < 0
                {
                    return Err(Error::last_os_error());
                }

                let [rd, wr] = pipe.assume_init();

                // ignore errno
                if CUSTOM_PIPE_CAP != DEFAULT_PIPE_CAP {
                    libc::fcntl(wr, libc::F_SETPIPE_SZ, CUSTOM_PIPE_CAP);
                }

                Ok(Pipe(rd, wr))
            }
        }
    }

    #[inline]
    fn splice_n(r: RawFd, w: RawFd, n: usize) -> isize {
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
    fn handle_wouldblock(is_wouldblock: &mut bool) -> Error {
        use libc::{EWOULDBLOCK, EAGAIN};
        let err = Error::last_os_error();
        match err.raw_os_error() {
            Some(e) if e == EWOULDBLOCK || e == EAGAIN => {
                *is_wouldblock = true;
                ErrorKind::WouldBlock.into()
            }
            _ => err,
        }
    }

    impl Require for CopyBuffer<Pipe> {
        fn new() -> Result<Self> {
            let pipe = Pipe::create()?;
            Ok(CopyBuffer {
                read_done: false,
                need_flush: false,
                pos: 0,
                cap: 0,
                amt: 0,
                buf: pipe,
            })
        }

        fn poll_read_tcp(
            &mut self,
            cx: &mut Context<'_>,
            stream: &mut TcpStream,
        ) -> Poll<Result<usize>> {
            loop {
                ready!(stream.poll_read_ready(cx))?;

                let mut is_wouldblock = false;
                let res =
                    stream.try_io(Interest::READABLE, || {
                        match splice_n(
                            stream.as_raw_fd(),
                            self.buf.1,
                            DEFAULT_PIPE_CAP,
                        ) {
                            x if x >= 0 => Ok(x as usize),
                            _ => Err(handle_wouldblock(&mut is_wouldblock)),
                        }
                    });

                if !is_wouldblock {
                    return Poll::Ready(res);
                }
            }
        }

        fn poll_write_tcp(
            &mut self,
            cx: &mut Context<'_>,
            stream: &mut TcpStream,
        ) -> Poll<Result<usize>> {
            loop {
                ready!(stream.poll_write_ready(cx)?);

                let mut is_wouldblock = false;
                let res =
                    stream.try_io(Interest::WRITABLE, || {
                        match splice_n(
                            self.buf.0,
                            stream.as_raw_fd(),
                            self.cap - self.pos,
                        ) {
                            x if x >= 0 => Ok(x as usize),
                            _ => Err(handle_wouldblock(&mut is_wouldblock)),
                        }
                    });

                if !is_wouldblock {
                    return Poll::Ready(res);
                }
            }
        }

        fn poll_flush_tcp(
            &mut self,
            cx: &mut Context<'_>,
            stream: &mut TcpStream,
        ) -> Poll<Result<()>> {
            Pin::new(stream).poll_flush(cx)
        }
    }
}
