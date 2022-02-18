use std::pin::Pin;
use std::task::{Poll, Context};
use std::future::Future;
use std::time::Duration;
use std::io::{Result, ErrorKind};

use tokio::time::Sleep;

use pin_project::pin_project;

#[allow(clippy::large_enum_variant)]
#[pin_project(project = DelayP)]
enum Delay {
    Some(#[pin] Sleep),

    None,
}

#[pin_project]
pub struct Timeout<T> {
    #[pin]
    value: T,

    #[pin]
    delay: Delay,
}

impl<T: Future> Future for Timeout<T> {
    type Output = Result<T::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        use Poll::{Pending, Ready};

        let this = self.project();

        if let Ready(v) = this.value.poll(cx) {
            return Ready(Ok(v));
        }

        if let DelayP::Some(delay) = this.delay.project() {
            let delay: Pin<&mut Sleep> = delay;
            if delay.poll(cx).is_ready() {
                return Ready(Err(ErrorKind::TimedOut.into()));
            }
        }

        Pending
    }
}

// timeout = 0 means never timeout
// instead of timeout immediately
pub fn timeoutfut<F: Future>(future: F, timeout: usize) -> Timeout<F> {
    use tokio::time::sleep;
    let delay = match timeout {
        0 => Delay::None,
        x => Delay::Some(sleep(Duration::from_secs(x as u64))),
    };
    Timeout {
        value: future,
        delay,
    }
}
