//! Statistic impl.  

use std::ops::AddAssign;
use std::io::{Result, IoSlice};
use std::pin::Pin;
use std::task::{Poll, Context};

use tokio::io::{ReadBuf, AsyncRead, AsyncWrite};

/// A wrapper to count written bytes.
pub struct StatStream<T, U> {
    pub io: T,
    pub stat: U,
}

impl<T, U> StatStream<T, U> {
    pub const fn new(io: T, stat: U) -> Self {
        Self { io, stat }
    }
}

impl<T, U> AsyncRead for StatStream<T, U>
where
    T: AsyncRead + Unpin,
    U: Unpin,
{
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.get_mut().io).poll_read(cx, buf)
    }
}

impl<T, U> AsyncWrite for StatStream<T, U>
where
    T: AsyncWrite + Unpin,
    U: AddAssign<usize> + Unpin,
{
    #[inline]
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        let this = self.get_mut();

        match Pin::new(&mut this.io).poll_write(cx, buf) {
            Poll::Ready(Ok(n)) => {
                this.stat += n;
                Poll::Ready(Ok(n))
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }

    #[inline]
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.get_mut().io).poll_flush(cx)
    }

    #[inline]
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.get_mut().io).poll_shutdown(cx)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        self.io.is_write_vectored()
    }

    #[inline]
    fn poll_write_vectored(self: Pin<&mut Self>, cx: &mut Context<'_>, iovec: &[IoSlice<'_>]) -> Poll<Result<usize>> {
        let this = self.get_mut();

        match Pin::new(&mut this.io).poll_write_vectored(cx, iovec) {
            Poll::Ready(Ok(n)) => {
                this.stat += n;
                Poll::Ready(Ok(n))
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}
