//! Peek impl.

use std::io::{Result, IoSlice};
use std::pin::Pin;
use std::future::Future;
use std::task::{Poll, Context, ready};

use tokio::io::{ReadBuf, AsyncRead, AsyncWrite};

/// A wrapper to inspect data without consuming underlying buffer.
pub struct PeekStream<T, U> {
    rd: usize,
    wr: usize,
    pub io: T,
    pub buf: U,
}

impl<T, U> PeekStream<T, U> {
    /// Create with provided buffer.
    pub const fn new(io: T, buf: U) -> Self {
        Self { io, rd: 0, wr: 0, buf }
    }

    /// Create and allocate memory on heap.
    pub fn new_alloc(io: T, n: usize) -> PeekStream<T, Box<[u8]>> {
        PeekStream {
            io,
            rd: 0,
            wr: 0,
            buf: vec![0; n].into_boxed_slice(),
        }
    }
}

impl<T, U> PeekStream<T, U>
where
    U: AsRef<[u8]> + AsMut<[u8]>,
{
    /// Return filled slice.  
    #[inline]
    pub fn filled_slice(&self) -> &[u8] {
        &self.buf.as_ref()[self.rd..self.wr]
    }

    /// Return unfilled mutable slice.
    #[inline]
    pub fn unfilled_slice(&mut self) -> &mut [u8] {
        &mut self.buf.as_mut()[self.wr..]
    }

    /// Buffer capacity.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.buf.as_ref().len()
    }

    /// Filled(unread) bytes.
    #[inline]
    pub fn filled(&self) -> usize {
        self.wr - self.rd
    }

    /// Unfilled(unwrite) bytes.
    #[inline]
    pub fn unfilled(&self) -> usize {
        self.buf.as_ref().len() - self.wr
    }

    #[inline]
    fn try_reset(&mut self) {
        if self.filled() == 0 {
            self.rd = 0;
            self.wr = 0;
        }
    }
}

struct Peek<'a, T, U> {
    pk: &'a mut PeekStream<T, U>,
}

impl<T, U> Future for Peek<'_, T, U>
where
    T: AsyncRead + Unpin,
    U: AsRef<[u8]> + AsMut<[u8]> + Unpin,
{
    type Output = Result<usize>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        let mut buf = ReadBuf::new(&mut this.pk.buf.as_mut()[this.pk.wr..]);

        ready!(Pin::new(&mut this.pk.io).poll_read(cx, &mut buf))?;

        let n = buf.filled().len();
        this.pk.wr += n;

        Poll::Ready(Ok(n))
    }
}

impl<T, U> PeekStream<T, U>
where
    T: AsyncRead + Unpin,
    U: AsRef<[u8]> + AsMut<[u8]> + Unpin,
{
    /// Peek bytes.
    pub async fn peek(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.try_reset();

        if self.unfilled() > 0 {
            Peek { pk: self }.await?;
        }

        let len = std::cmp::min(self.filled(), buf.len());
        let (left, _) = buf.split_at_mut(len);
        left.copy_from_slice(&self.filled_slice()[..self.rd + len]);
        Ok(len)
    }

    /// Peek exact n bytes, fill the provided buffer.
    pub async fn peek_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        assert!(self.capacity() >= buf.len());
        self.try_reset();

        let len = buf.len();
        let mut required = len.saturating_sub(self.filled());

        while required > 0 {
            let n = Peek { pk: self }.await?;
            required = required.saturating_sub(n);
        }

        let (left, _) = buf.split_at_mut(len);
        left.copy_from_slice(&self.filled_slice()[..self.rd + len]);
        Ok(())
    }
}

impl<T, U> AsyncRead for PeekStream<T, U>
where
    T: AsyncRead + Unpin,
    U: AsRef<[u8]> + Unpin,
{
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<Result<()>> {
        let this = self.get_mut();

        if this.rd == this.wr {
            Pin::new(&mut this.io).poll_read(cx, buf)
        } else {
            let extra = this.buf.as_ref();
            let len = std::cmp::min(this.wr - this.rd, buf.remaining());
            buf.put_slice(&extra[this.rd..this.rd + len]);
            this.rd += len;
            Poll::Ready(Ok(()))
        }
    }
}

impl<T, U> AsyncWrite for PeekStream<T, U>
where
    T: AsyncWrite + Unpin,
    U: Unpin,
{
    #[inline]
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        Pin::new(&mut self.get_mut().io).poll_write(cx, buf)
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
        Pin::new(&mut self.get_mut().io).poll_write_vectored(cx, iovec)
    }
}
