use std::io::Result;
use std::pin::Pin;
use std::task::{Poll, Context};

use tokio::io::ReadBuf;
use tokio::io::{AsyncRead, AsyncWrite};

use super::{CopyBuffer, AsyncIOBuf};

impl<B, S> AsyncIOBuf for CopyBuffer<B, S>
where
    B: AsMut<[u8]>,
    S: AsyncRead + AsyncWrite + Unpin,
{
    type Stream = S;

    #[inline]
    fn poll_read_buf(
        &mut self,
        cx: &mut Context<'_>,
        stream: &mut Self::Stream,
    ) -> Poll<Result<usize>> {
        let mut buf = ReadBuf::new(self.buf.as_mut());
        Pin::new(stream)
            .poll_read(cx, &mut buf)
            .map_ok(|_| buf.filled().len())
    }

    #[inline]
    fn poll_write_buf(
        &mut self,
        cx: &mut Context<'_>,
        stream: &mut Self::Stream,
    ) -> Poll<Result<usize>> {
        Pin::new(stream).poll_write(cx, &self.buf.as_mut()[self.pos..self.cap])
    }

    #[inline]
    fn poll_flush_buf(
        &mut self,
        cx: &mut Context<'_>,
        stream: &mut Self::Stream,
    ) -> Poll<Result<()>> {
        Pin::new(stream).poll_flush(cx)
    }
}
