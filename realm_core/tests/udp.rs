use std::net::SocketAddr;
use std::time::Duration;

use tokio::net::UdpSocket;
use tokio::time::sleep;

use realm_core::udp::run_udp;
use realm_core::endpoint::{Endpoint, RemoteAddr};

#[tokio::test]
async fn udp() {
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

    tokio::spawn(run_udp(endpoint));

    let task1 = async {
        sleep(Duration::from_millis(500)).await;

        let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();

        let mut buf = vec![0; 32];
        let peer: SocketAddr = "127.0.0.1:10000".parse().unwrap();

        for _ in 0..20 {
            socket.send_to(b"Ping Ping Ping", &peer).await.unwrap();
            let (n, peer2) = socket.recv_from(&mut buf).await.unwrap();
            assert_eq!(peer, peer2);
            log::debug!("a got: {:?}", std::str::from_utf8(&buf[..n]).unwrap());
            assert_eq!(b"Pong Pong Pong", &buf[..n]);
        }
    };

    let task2 = async {
        let socket = UdpSocket::bind("127.0.0.1:20000").await.unwrap();

        let mut buf = vec![0; 32];

        for _ in 0..20 {
            let (n, peer) = socket.recv_from(&mut buf).await.unwrap();
            log::debug!("b got: {:?}", std::str::from_utf8(&buf[..n]).unwrap());
            assert_eq!(b"Ping Ping Ping", &buf[..n]);
            socket.send_to(b"Pong Pong Pong", peer).await.unwrap();
        }
    };

    tokio::join!(task1, task2);
}
