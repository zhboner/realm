use std::error::Error;
use std::net::SocketAddr;

use futures::future::try_join;
use futures::FutureExt;
use tokio;
use tokio::io;
use tokio::net;

pub async fn start(
    client_socket: SocketAddr,
    remote_socket: SocketAddr,
) -> Result<(), Box<dyn Error>> {
    let mut listener = net::TcpListener::bind(&client_socket).await?;
    println!("Listening on: {}", client_socket);

    while let Ok((inbound, _)) = listener.accept().await {
        let transfer = transfer(inbound, remote_socket.clone()).map(|r| {
            if let Err(_) = r {
                return;
            }
        });
        tokio::spawn(transfer);
    }
    Ok(())
}

async fn transfer(
    mut inbound: net::TcpStream,
    remote_socket: SocketAddr,
) -> Result<(), Box<dyn Error>> {
    let mut outbound = net::TcpStream::connect(remote_socket).await?;
    let (mut ri, mut wi) = inbound.split();
    let (mut ro, mut wo) = outbound.split();

    let client_to_server = io::copy(&mut ri, &mut wo);
    let server_to_client = io::copy(&mut ro, &mut wi);

    try_join(client_to_server, server_to_client).await?;

    Ok(())
}
