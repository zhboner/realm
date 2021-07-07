use std::ops::Drop;
use tokio::io;
use super::utils;

pub struct Pipe(pub i32, pub i32);

impl Drop for Pipe {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.0);
            libc::close(self.1);
        }
    }
}

impl Pipe {
    pub fn create() -> io::Result<Self> {
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

pub fn splice_n(r: i32, w: i32, n: usize) -> isize {
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

pub fn is_wouldblock() -> bool {
    use libc::{EAGAIN, EWOULDBLOCK};
    let errno = unsafe { *libc::__errno_location() };
    errno == EWOULDBLOCK || errno == EAGAIN
}
