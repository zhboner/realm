use std::io::Result;
use std::pin::Pin;
use std::task::{Context, Poll, ready};
use std::future::Future;

use tokio::io::{AsyncRead, AsyncWrite};

use super::{AsyncIOBuf, CopyBuffer};

enum TransferState<B, S> {
    Running(CopyBuffer<B, S>),
    ShuttingDown,
    Done,
}

struct BidiCopy<'a, B, S>
where
    B: Unpin,
    S: AsyncRead + AsyncWrite + Unpin,
    CopyBuffer<B, S>: AsyncIOBuf + Unpin,
{
    a: &'a mut <CopyBuffer<B, S> as AsyncIOBuf>::Stream,
    b: &'a mut <CopyBuffer<B, S> as AsyncIOBuf>::Stream,
    a_to_b: TransferState<B, S>,
    b_to_a: TransferState<B, S>,
    ab_amt: &'a mut u64,
    ba_amt: &'a mut u64,
}

fn transfer_one_direction<B, S>(
    cx: &mut Context<'_>,
    state: &mut TransferState<B, S>,
    r: &mut <CopyBuffer<B, S> as AsyncIOBuf>::Stream,
    w: &mut <CopyBuffer<B, S> as AsyncIOBuf>::Stream,
    amt: &mut u64,
) -> Poll<Result<()>>
where
    B: Unpin,
    S: AsyncRead + AsyncWrite + Unpin,
    CopyBuffer<B, S>: AsyncIOBuf,
{
    loop {
        match state {
            TransferState::Running(buf) => {
                ready!(buf.poll_copy(cx, r, w, amt))?;

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

impl<'a, B, S> Future for BidiCopy<'a, B, S>
where
    B: Unpin,
    S: AsyncRead + AsyncWrite + Unpin,
    CopyBuffer<B, S>: AsyncIOBuf + Unpin,
{
    type Output = Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Unpack self into mut refs to each field to avoid borrow check issues.
        let BidiCopy {
            a,
            b,
            a_to_b,
            b_to_a,
            ab_amt,
            ba_amt,
        } = self.get_mut();

        let a_to_b = transfer_one_direction(cx, a_to_b, a, b, ab_amt)?;
        let b_to_a = transfer_one_direction(cx, b_to_a, b, a, ba_amt)?;

        // It is not a problem if ready! returns early because transfer_one_direction for the
        // other direction will keep returning TransferState::Done(count) in future calls to poll
        ready!(a_to_b);
        ready!(b_to_a);

        Poll::Ready(Ok(()))
    }
}

pub async fn bidi_copy_buf<B, S>(
    a: &mut <CopyBuffer<B, S> as AsyncIOBuf>::Stream,
    b: &mut <CopyBuffer<B, S> as AsyncIOBuf>::Stream,
    a_to_b_buf: CopyBuffer<B, S>,
    b_to_a_buf: CopyBuffer<B, S>,
) -> (Result<()>, u64, u64)
where
    B: Unpin,
    S: AsyncRead + AsyncWrite + Unpin,
    CopyBuffer<B, S>: AsyncIOBuf + Unpin,
{
    let a_to_b = TransferState::Running(a_to_b_buf);
    let b_to_a = TransferState::Running(b_to_a_buf);

    let mut ab_amt = 0;
    let mut ba_amt = 0;

    let res = BidiCopy {
        a,
        b,
        a_to_b,
        b_to_a,
        ab_amt: &mut ab_amt,
        ba_amt: &mut ba_amt,
    }
    .await;

    (res, ab_amt, ba_amt)
}
