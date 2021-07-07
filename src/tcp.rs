use futures::try_join;

use tokio::io::{self, AsyncWriteExt};
use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;

use crate::utils::{self, RemoteAddr};

pub async fn proxy(
    mut inbound: TcpStream,
    remote: RemoteAddr,
) -> io::Result<()> {
    let mut outbound = TcpStream::connect(remote.to_sockaddr().await?).await?;
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
async fn copy(r: &mut ReadHalf<'_>, w: &mut WriteHalf<'_>) -> io::Result<()> {
    use std::os::unix::prelude::AsRawFd;
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

#[cfg(target_os = "linux")]
struct Pipe(i32, i32);

#[cfg(target_os = "linux")]
impl Drop for Pipe {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.0);
            libc::close(self.1);
        }
    }
}

#[cfg(target_os = "linux")]
impl Pipe {
    fn create() -> io::Result<Self> {
        use libc::{c_int, O_NONBLOCK};
        let mut pipes = std::mem::MaybeUninit::<[c_int; 2]>::uninit();
        unsafe {
            if libc::pipe2(pipes.as_mut_ptr() as *mut c_int, O_NONBLOCK) < 0 {
                return Err(utils::new_io_err("failed to create a pipe"));
            }
            Ok(Pipe(pipes.assume_init()[0], pipes.assume_init()[1]))
        }
    }
}

#[cfg(target_os = "linux")]
fn splice_n(r: i32, w: i32, n: usize) -> isize {
    use libc::{loff_t, SPLICE_F_MOVE, SPLICE_F_NONBLOCK};
    unsafe {
        libc::splice(
            r,
            0 as *mut loff_t,
            w,
            0 as *mut loff_t,
            n,
            SPLICE_F_MOVE | SPLICE_F_NONBLOCK,
        )
    }
}

#[cfg(target_os = "linux")]
fn is_wouldblock() -> bool {
    use libc::{EAGAIN, EWOULDBLOCK};
    let errno = unsafe { *libc::__errno_location() };
    errno == EWOULDBLOCK || errno == EAGAIN
}
