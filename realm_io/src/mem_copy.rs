use std::io::Result;
use std::pin::Pin;
use std::task::{Poll, Context};

use tokio::io::ReadBuf;
use tokio::io::{AsyncRead, AsyncWrite};

use super::{CopyBuffer, AsyncIOBuf};
use super::bidi_copy_buf;

impl<B, SR, SW> AsyncIOBuf for CopyBuffer<B, SR, SW>
where
    B: AsMut<[u8]>,
    SR: AsyncRead + AsyncWrite + Unpin,
    SW: AsyncRead + AsyncWrite + Unpin,
{
    type StreamR = SR;
    type StreamW = SW;

    #[inline]
    fn poll_read_buf(
        &mut self,
        cx: &mut Context<'_>,
        stream: &mut Self::StreamR,
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
        stream: &mut Self::StreamW,
    ) -> Poll<Result<usize>> {
        Pin::new(stream).poll_write(cx, &self.buf.as_mut()[self.pos..self.cap])
    }

    #[inline]
    fn poll_flush_buf(
        &mut self,
        cx: &mut Context<'_>,
        stream: &mut Self::StreamW,
    ) -> Poll<Result<()>> {
        Pin::new(stream).poll_flush(cx)
    }
}

pub async fn bidi_copy<A, B>(a: &mut A, b: &mut B) -> (Result<()>, u64, u64)
where
    A: AsyncRead + AsyncWrite + Unpin,
    B: AsyncRead + AsyncWrite + Unpin,
{
    let a_to_b_buf = CopyBuffer::new(vec![0u8; 0x2000]);
    let b_to_a_buf = CopyBuffer::new(vec![0u8; 0x2000]);
    bidi_copy_buf(a, b, a_to_b_buf, b_to_a_buf).await
}
