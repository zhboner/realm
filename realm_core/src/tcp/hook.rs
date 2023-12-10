use std::io::{Result, Error, ErrorKind};

use tokio::net::TcpStream;
use realm_hook::pre_conn::{self, first_pkt_len, decide_remote_idx};

use crate::endpoint::RemoteAddr;

pub async fn pre_connect_hook<'a>(
    local: &mut TcpStream,
    raddr: &'a RemoteAddr,
    extra_raddrs: &'a [RemoteAddr],
) -> Result<&'a RemoteAddr> {
    if !pre_conn::is_loaded() {
        return Ok(raddr);
    }

    let len = first_pkt_len() as usize;
    let mut buf = Vec::<u8>::new();

    if len != 0 {
        buf.resize(len, 0);
        while local.peek(&mut buf).await? < len {}
    }

    let mut idx = extra_raddrs.len() as i32;
    idx = decide_remote_idx(idx, buf.as_ptr());

    match idx {
        0 => Ok(raddr),
        i if i >= 1 && i <= idx => Ok(&extra_raddrs[i as usize - 1]),
        _ => Err(Error::new(ErrorKind::Other, "rejected by pre-connect hook")),
    }
}
