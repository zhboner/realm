use std::io::{Result, Error};
use std::pin::Pin;
use std::task::{Poll, Context};
use std::os::unix::io::RawFd;

use tokio::io::{AsyncRead, AsyncWrite};

use crate::{CopyBuffer, AsyncIOBuf, AsyncRawIO};
use crate::bidi_copy_buf;

/// Unix pipe.
pub struct Pipe(RawFd, RawFd);

impl Pipe {
    pub fn new() -> Result<Self> {
        use libc::{c_int, O_NONBLOCK};
        use pipe_ctl::DF_PIPE_SIZE;

        let mut pipe = std::mem::MaybeUninit::<[c_int; 2]>::uninit();
        unsafe {
            if libc::pipe2(pipe.as_mut_ptr() as *mut c_int, O_NONBLOCK) < 0 {
                return Err(Error::last_os_error());
            }

            let [rd, wr] = pipe.assume_init();

            // ignore errno
            if pipe_size() != DF_PIPE_SIZE {
                libc::fcntl(wr, libc::F_SETPIPE_SZ, pipe_size());
            }

            Ok(Pipe(rd, wr))
        }
    }
}

impl Drop for Pipe {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.0);
            libc::close(self.1);
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

impl<SR, SW> AsyncIOBuf for CopyBuffer<Pipe, SR, SW>
where
    SR: AsyncRead + AsyncWrite + AsyncRawIO + Unpin,
    SW: AsyncRead + AsyncWrite + AsyncRawIO + Unpin,
{
    type StreamR = SR;
    type StreamW = SW;

    fn poll_read_buf(&mut self, cx: &mut Context<'_>, stream: &mut Self::StreamR) -> Poll<Result<usize>> {
        stream.poll_read_raw(cx, || splice_n(stream.as_raw_fd(), self.buf.1, usize::MAX))
    }

    fn poll_write_buf(&mut self, cx: &mut Context<'_>, stream: &mut Self::StreamW) -> Poll<Result<usize>> {
        stream.poll_write_raw(cx, || splice_n(self.buf.0, stream.as_raw_fd(), self.cap - self.pos))
    }

    fn poll_flush_buf(&mut self, cx: &mut Context<'_>, stream: &mut Self::StreamW) -> Poll<Result<()>> {
        Pin::new(stream).poll_flush(cx)
    }
}

impl<'a, SR, SW> AsyncIOBuf for CopyBuffer<&'a mut Pipe, SR, SW>
where
    SR: AsyncRead + AsyncWrite + AsyncRawIO + Unpin,
    SW: AsyncRead + AsyncWrite + AsyncRawIO + Unpin,
{
    type StreamR = SR;
    type StreamW = SW;

    fn poll_read_buf(&mut self, cx: &mut Context<'_>, stream: &mut Self::StreamR) -> Poll<Result<usize>> {
        stream.poll_read_raw(cx, || splice_n(stream.as_raw_fd(), self.buf.1, usize::MAX))
    }

    fn poll_write_buf(&mut self, cx: &mut Context<'_>, stream: &mut Self::StreamW) -> Poll<Result<usize>> {
        stream.poll_write_raw(cx, || splice_n(self.buf.0, stream.as_raw_fd(), self.cap - self.pos))
    }

    fn poll_flush_buf(&mut self, cx: &mut Context<'_>, stream: &mut Self::StreamW) -> Poll<Result<()>> {
        Pin::new(stream).poll_flush(cx)
    }
}

/// Copy data bidirectionally between two streams with pipe.
pub async fn bidi_zero_copy<A, B>(a: &mut A, b: &mut B) -> Result<(u64, u64)>
where
    A: AsyncRead + AsyncWrite + AsyncRawIO + Unpin,
    B: AsyncRead + AsyncWrite + AsyncRawIO + Unpin,
{
    let a_to_b_buf = CopyBuffer::new(Pipe::new()?);
    let b_to_a_buf = CopyBuffer::new(Pipe::new()?);
    bidi_copy_buf(a, b, a_to_b_buf, b_to_a_buf).await
}

mod pipe_ctl {
    pub const DF_PIPE_SIZE: usize = 16 * 0x1000;
    static mut PIPE_SIZE: usize = DF_PIPE_SIZE;

    /// Get pipe capacity.
    #[inline]
    pub fn pipe_size() -> usize {
        unsafe { PIPE_SIZE }
    }

    /// Set pipe capacity.
    #[inline]
    pub fn set_pipe_size(n: usize) {
        unsafe { PIPE_SIZE = n }
    }
}

pub use pipe_ctl::{pipe_size, set_pipe_size};
