use std::io::Result;
use tokio::net::TcpStream;

#[inline]
pub async fn run_relay(mut local: TcpStream, mut remote: TcpStream) -> Result<()> {
    #[cfg(all(target_os = "linux"))]
    {
        let (res, _, _) = realm_io::bidi_zero_copy(&mut local, &mut remote).await;
        res
    }

    #[cfg(not(target_os = "linux"))]
    {
        let (res, _, _) = realm_io::bidi_copy(&mut local, &mut remote).await;
        res
    }
}
