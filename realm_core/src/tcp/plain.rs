use std::io::Result;
use tokio::net::TcpStream;

const EINVAL: i32 = 22;

#[inline]
pub async fn run_relay(mut local: TcpStream, mut remote: TcpStream) -> Result<()> {
    #[cfg(all(target_os = "linux"))]
    {
        match realm_io::bidi_zero_copy(&mut local, &mut remote).await {
            Ok(()) => Ok(()),
            Err(ref e) if e.raw_os_error().map_or(false, |ec| ec == EINVAL) => {
                realm_io::bidi_copy(&mut local, &mut remote).await
            }
            Err(e) => Err(e),
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        realm_io::bidi_copy(&mut local, &mut remote).await
    }
}
