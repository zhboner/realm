use std::net::SocketAddr;

use tokio::net::{TcpStream, TcpListener};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use realm::relay::run_tcp;
use realm::utils::Endpoint;
use realm::utils::timeoutfut;

#[tokio::test]
async fn tcp() {
    env_logger::init();
    let endpoint = Endpoint {
        listen: "127.0.0.1:10000".parse().unwrap(),
        remote: "127.0.0.1:20000".parse::<SocketAddr>().unwrap().into(),
        opts: Default::default(),
    };

    tokio::spawn(async {
        let mut stream = TcpStream::connect("127.0.0.1:10000").await.unwrap();

        let mut buf = vec![0; 32];

        for _ in 0..20 {
            stream.write(b"Ping Ping Ping").await.unwrap();
            let n = stream.read(&mut buf).await.unwrap();
            log::debug!("a got: {:?}", std::str::from_utf8(&buf[..n]).unwrap());
            assert_eq!(b"Pong Pong Pong", &buf[..n]);
        }
    });

    tokio::spawn(async {
        let lis = TcpListener::bind("127.0.0.1:20000").await.unwrap();
        let (mut stream, _) = lis.accept().await.unwrap();

        let mut buf = vec![0; 32];

        for _ in 0..20 {
            let n = stream.read(&mut buf).await.unwrap();
            log::debug!("b got: {:?}", std::str::from_utf8(&buf[..n]).unwrap());
            assert_eq!(b"Ping Ping Ping", &buf[..n]);
            stream.write(b"Pong Pong Pong").await.unwrap();
        }
    });

    let _ = timeoutfut(run_tcp((&endpoint).into()), 3).await;
}
