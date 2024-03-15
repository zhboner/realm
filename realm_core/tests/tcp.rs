use std::net::SocketAddr;
use std::time::Duration;

use tokio::net::{TcpStream, TcpListener};
use tokio::time::sleep;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use realm_core::tcp::run_tcp;
use realm_core::endpoint::{Endpoint, RemoteAddr};

#[tokio::test]
async fn tcp() {
    env_logger::init();
    let endpoint = Endpoint {
        laddr: "127.0.0.1:10000".parse().unwrap(),
        raddr: "127.0.0.1:20000"
            .parse::<SocketAddr>()
            .map(RemoteAddr::SocketAddr)
            .unwrap(),
        conn_opts: Default::default(),
        bind_opts: Default::default(),
        extra_raddrs: Vec::new(),
    };

    tokio::spawn(run_tcp(endpoint));

    let task1 = async {
        sleep(Duration::from_millis(500)).await;
        let mut stream = TcpStream::connect("127.0.0.1:10000").await.unwrap();

        let mut buf = vec![0; 32];

        for _ in 0..20 {
            stream.write(b"Ping Ping Ping").await.unwrap();
            let n = stream.read(&mut buf).await.unwrap();
            log::debug!("a got: {:?}", std::str::from_utf8(&buf[..n]).unwrap());
            assert_eq!(b"Pong Pong Pong", &buf[..n]);
        }
    };

    let task2 = async {
        let lis = TcpListener::bind("127.0.0.1:20000").await.unwrap();
        let (mut stream, _) = lis.accept().await.unwrap();

        let mut buf = vec![0; 32];

        for _ in 0..20 {
            let n = stream.read(&mut buf).await.unwrap();
            log::debug!("b got: {:?}", std::str::from_utf8(&buf[..n]).unwrap());
            assert_eq!(b"Ping Ping Ping", &buf[..n]);
            stream.write(b"Pong Pong Pong").await.unwrap();
        }
    };

    tokio::join!(task1, task2);
}
