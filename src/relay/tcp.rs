use futures::try_join;

use tokio::io::{self, AsyncWriteExt};
use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;

use super::types::RemoteAddr;

pub async fn proxy(
    mut inbound: TcpStream,
    remote: RemoteAddr,
) -> io::Result<()> {
    let mut outbound =
        TcpStream::connect(remote.into_sockaddr().await?).await?;
    inbound.set_nodelay(true)?;
    outbound.set_nodelay(true)?;
    let (mut ri, mut wi) = inbound.split();
    let (mut ro, mut wo) = outbound.split();

    let _ = try_join!(copy(&mut ri, &mut wo), copy(&mut ro, &mut wi));

    Ok(())
}

const BUFFERSIZE: usize = if cfg!(not(target_os = "linux")) {
    0x4000 // 16k read/write buffer
} else {
    0x10000 // 64k pipe buffer
};

#[cfg(not(target_os = "linux"))]
async fn copy(r: &mut ReadHalf<'_>, w: &mut WriteHalf<'_>) -> io::Result<()> {
    use io::AsyncReadExt;
    let mut buf = vec![0u8; BUFFERSIZE];
    let mut n: usize;
    loop {
        n = r.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        w.write(&buf[..n]).await?;
        w.flush().await?;
    }
    w.shutdown().await?;
    Ok(())
}

// zero copy
#[cfg(target_os = "linux")]
use crate::relay::zero_copy;

#[cfg(target_os = "linux")]
async fn copy(r: &mut ReadHalf<'_>, w: &mut WriteHalf<'_>) -> io::Result<()> {
    use std::os::unix::prelude::AsRawFd;
    use zero_copy::{Pipe, splice_n, is_wouldblock};
    // create pipe
    let pipe = Pipe::create()?;
    let (rpipe, wpipe) = (pipe.0, pipe.1);
    // get raw fd
    let rfd = r.as_ref().as_raw_fd();
    let wfd = w.as_ref().as_raw_fd();
    let mut n: usize = 0;
    let mut done = false;

    'LOOP: loop {
        // read until the socket buffer is empty
        // or the pipe is filled
        r.as_ref().readable().await?;
        while n < BUFFERSIZE {
            match splice_n(rfd, wpipe, BUFFERSIZE - n) {
                x if x > 0 => n += x as usize,
                x if x == 0 => {
                    done = true;
                    break;
                }
                x if x < 0 && is_wouldblock() => break,
                _ => break 'LOOP,
            }
        }
        // write until the pipe is empty
        while n > 0 {
            w.as_ref().writable().await?;
            match splice_n(rpipe, wfd, n) {
                x if x > 0 => n -= x as usize,
                x if x < 0 && is_wouldblock() => {
                    // clear readiness (EPOLLOUT)
                    let _ = r.as_ref().try_write(&[0u8; 0]);
                }
                _ => break 'LOOP,
            }
        }
        // complete
        if done {
            break;
        }
        // clear readiness (EPOLLIN)
        let _ = r.as_ref().try_read(&mut [0u8; 0]);
    }

    w.shutdown().await?;
    Ok(())
}
