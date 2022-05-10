use std::io::Result;
use std::pin::Pin;
use std::task::{Context, Poll, ready};
use std::future::Future;

use tokio::io::{AsyncRead, AsyncWrite};

use super::{AsyncIOBuf, CopyBuffer};

enum TransferState<B, SR, SW> {
    Running(CopyBuffer<B, SR, SW>),
    ShuttingDown,
    Done,
}

fn transfer<B, SL, SR>(
    cx: &mut Context<'_>,
    state: &mut TransferState<B, SL, SR>,
    r: &mut <CopyBuffer<B, SL, SR> as AsyncIOBuf>::StreamR,
    w: &mut <CopyBuffer<B, SL, SR> as AsyncIOBuf>::StreamW,
) -> Poll<Result<()>>
where
    B: Unpin,
    SL: AsyncRead + AsyncWrite + Unpin,
    SR: AsyncRead + AsyncWrite + Unpin,
    CopyBuffer<B, SL, SR>: AsyncIOBuf,
    CopyBuffer<B, SR, SL>: AsyncIOBuf,
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

fn transfer2<B, SL, SR>(
    cx: &mut Context<'_>,
    state: &mut TransferState<B, SR, SL>, // reverse
    r: &mut <CopyBuffer<B, SL, SR> as AsyncIOBuf>::StreamW,
    w: &mut <CopyBuffer<B, SL, SR> as AsyncIOBuf>::StreamR,
) -> Poll<Result<()>>
where
    B: Unpin,
    SL: AsyncRead + AsyncWrite + Unpin,
    SR: AsyncRead + AsyncWrite + Unpin,
    CopyBuffer<B, SL, SR>: AsyncIOBuf,
    CopyBuffer<B, SR, SL>: AsyncIOBuf,
{
    // type equality constraints will save this (one day)!
    let r: &mut <CopyBuffer<B, SR, SL> as AsyncIOBuf>::StreamR = unsafe { std::mem::transmute(r) };
    let w: &mut <CopyBuffer<B, SR, SL> as AsyncIOBuf>::StreamW = unsafe { std::mem::transmute(w) };
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

struct BidiCopy<'a, B, SL, SR>
where
    B: Unpin,
    SL: AsyncRead + AsyncWrite + Unpin,
    SR: AsyncRead + AsyncWrite + Unpin,
    CopyBuffer<B, SL, SR>: AsyncIOBuf,
    CopyBuffer<B, SR, SL>: AsyncIOBuf,
{
    a: &'a mut <CopyBuffer<B, SL, SR> as AsyncIOBuf>::StreamR,
    b: &'a mut <CopyBuffer<B, SL, SR> as AsyncIOBuf>::StreamW,
    a_to_b: TransferState<B, SL, SR>,
    b_to_a: TransferState<B, SR, SL>,
}

impl<'a, B, SL, SR> Future for BidiCopy<'a, B, SL, SR>
where
    B: Unpin,
    SL: AsyncRead + AsyncWrite + Unpin,
    SR: AsyncRead + AsyncWrite + Unpin,
    CopyBuffer<B, SL, SR>: AsyncIOBuf,
    CopyBuffer<B, SR, SL>: AsyncIOBuf,
{
    type Output = Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Unpack self into mut refs to each field to avoid borrow check issues.
        let BidiCopy { a, b, a_to_b, b_to_a } = self.get_mut();

        let a_to_b = transfer(cx, a_to_b, a, b)?;
        let b_to_a = transfer2::<B, SL, SR>(cx, b_to_a, b, a)?;

        // graceful shutdown
        #[cfg(not(feature = "brutal-shutdown"))]
        {
            ready!(a_to_b);
            ready!(b_to_a);
            Poll::Ready(Ok(()))
        }

        // brutal shutdown
        #[cfg(feature = "brutal-shutdown")]
        {
            if a_to_b.is_ready() || b_to_a.is_ready() {
                Poll::Ready(Ok(()))
            } else {
                Poll::Pending
            }
        }
    }
}

/// Copy data bidirectionally between two streams via two provided buffers.
///
/// Return transferred bytes no matter this operation succeeds or fails.
pub async fn bidi_copy_buf<B, SR, SW>(
    a: &mut <CopyBuffer<B, SR, SW> as AsyncIOBuf>::StreamR,
    b: &mut <CopyBuffer<B, SR, SW> as AsyncIOBuf>::StreamW,
    a_to_b_buf: CopyBuffer<B, SR, SW>,
    b_to_a_buf: CopyBuffer<B, SW, SR>,
) -> Result<()>
where
    B: Unpin,
    SR: AsyncRead + AsyncWrite + Unpin,
    SW: AsyncRead + AsyncWrite + Unpin,
    CopyBuffer<B, SR, SW>: AsyncIOBuf,
    CopyBuffer<B, SW, SR>: AsyncIOBuf,
{
    let a_to_b = TransferState::Running(a_to_b_buf);
    let b_to_a = TransferState::Running(b_to_a_buf);

    BidiCopy { a, b, a_to_b, b_to_a }.await
}
