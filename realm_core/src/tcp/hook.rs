use std::io::{Result, Error, ErrorKind};

use tokio::net::TcpStream;
use realm_hook::{first_pkt_len, decide_remote_idx};

use crate::endpoint::RemoteAddr;

pub async fn pre_connect_hook<'a>(
    local: &mut TcpStream,
    raddr: &'a RemoteAddr,
    extra_raddrs: &'a Vec<RemoteAddr>,
) -> Result<&'a RemoteAddr> {
    let len = first_pkt_len() as usize;
    let mut buf = Vec::<u8>::new();

    if len != 0 {
        buf.resize(len, 0);
        while local.peek(&mut buf).await? < len {}
    }

    let idx = decide_remote_idx(buf.as_ptr());

    match idx {
        0 => Ok(raddr),
        i if i >= 1 && i <= extra_raddrs.len() as i32 => Ok(&extra_raddrs[i as usize - 1]),
        _ => Err(Error::new(ErrorKind::Other, "rejected by pre-connect hook")),
    }
}
