use std::io::{Result, Error, ErrorKind};
use futures::try_join;

use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;

use crate::utils::RemoteAddr;

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(target_os = "linux")] {
        use zero_copy::copy;
        const BUFFER_SIZE: usize = 0x10000;
    } else {
        use normal_copy::copy;
        const BUFFER_SIZE: usize = 0x4000;
    }
}

pub async fn proxy(mut inbound: TcpStream, remote: RemoteAddr) -> Result<()> {
    let mut outbound =
        TcpStream::connect(remote.into_sockaddr().await?).await?;
    inbound.set_nodelay(true)?;
    outbound.set_nodelay(true)?;
    let (ri, wi) = inbound.split();
    let (ro, wo) = outbound.split();

    let _ = try_join!(copy(ri, wo), copy(ro, wi));

    Ok(())
}

#[cfg(not(target_os = "linux"))]
mod normal_copy {
    use super::*;
    pub async fn copy(mut r: ReadHalf<'_>, mut w: WriteHalf<'_>) -> Result<()> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let mut buf = vec![0u8; BUFFER_SIZE];
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
}

#[cfg(target_os = "linux")]
mod zero_copy {
    use super::*;
    use std::ops::Drop;
    use tokio::io::Interest;

    struct Pipe(pub i32, pub i32);

    impl Drop for Pipe {
        fn drop(&mut self) {
            unsafe {
                libc::close(self.0);
                libc::close(self.1);
            }
        }
    }

    impl Pipe {
        fn create() -> Result<Self> {
            use libc::{c_int, O_NONBLOCK};
            let mut pipes = std::mem::MaybeUninit::<[c_int; 2]>::uninit();
            unsafe {
                if libc::pipe2(pipes.as_mut_ptr() as *mut c_int, O_NONBLOCK) < 0
                {
                    return Err(Error::new(
                        ErrorKind::Unsupported,
                        "failed to create a pipe",
                    ));
                }
                Ok(Pipe(pipes.assume_init()[0], pipes.assume_init()[1]))
            }
        }
    }

    #[inline]
    fn splice_n(r: i32, w: i32, n: usize) -> isize {
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

    #[inline]
    fn is_wouldblock() -> bool {
        use libc::{EAGAIN, EWOULDBLOCK};
        let errno = unsafe { *libc::__errno_location() };
        errno == EWOULDBLOCK || errno == EAGAIN
    }

    #[inline]
    fn clear_readiness(x: &TcpStream, interest: Interest) {
        let _ = x.try_io(interest, || {
            Err(Error::new(ErrorKind::WouldBlock, "")) as Result<()>
        });
    }

    pub async fn copy(r: ReadHalf<'_>, mut w: WriteHalf<'_>) -> Result<()> {
        use std::os::unix::io::AsRawFd;
        use tokio::io::AsyncWriteExt;
        // init pipe
        let pipe = Pipe::create()?;
        let (rpipe, wpipe) = (pipe.0, pipe.1);
        // rw ref
        let rx = r.as_ref();
        let wx = w.as_ref();
        // rw raw fd
        let rfd = rx.as_raw_fd();
        let wfd = wx.as_raw_fd();
        // ctrl
        let mut n: usize = 0;
        let mut done = false;

        'LOOP: loop {
            // read until the socket buffer is empty
            // or the pipe is filled
            rx.readable().await?;
            while n < BUFFER_SIZE {
                match splice_n(rfd, wpipe, BUFFER_SIZE - n) {
                    x if x > 0 => n += x as usize,
                    x if x == 0 => {
                        done = true;
                        break;
                    }
                    x if x < 0 && is_wouldblock() => {
                        clear_readiness(rx, Interest::READABLE);
                        break;
                    }
                    _ => break 'LOOP,
                }
            }
            // write until the pipe is empty
            while n > 0 {
                wx.writable().await?;
                match splice_n(rpipe, wfd, n) {
                    x if x > 0 => n -= x as usize,
                    x if x < 0 && is_wouldblock() => {
                        clear_readiness(wx, Interest::WRITABLE)
                    }
                    _ => break 'LOOP,
                }
            }
            // complete
            if done {
                break;
            }
        }

        if done {
            w.shutdown().await?;
            Ok(())
        } else {
            Err(Error::new(ErrorKind::ConnectionReset, "connection reset"))
        }
    }
}
