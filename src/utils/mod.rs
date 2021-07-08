use tokio::io;

pub fn new_io_err(e: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Other, e)
}
