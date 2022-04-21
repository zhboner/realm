use std::net::SocketAddr;

use tokio::net::UdpSocket;

use realm::relay::run_udp;
use realm::utils::{Endpoint, ConnectOpts};
use realm::utils::timeoutfut;

#[tokio::test]
async fn udp() {
    env_logger::init();
    let endpoint = Endpoint {
        listen: "127.0.0.1:10000".parse().unwrap(),
        remote: "127.0.0.1:20000".parse::<SocketAddr>().unwrap().into(),
        conn_opts: ConnectOpts {
            udp_timeout: 20,
            ..Default::default()
        },
    };

    tokio::spawn(async {
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
    });

    tokio::spawn(async {
        let socket = UdpSocket::bind("127.0.0.1:20000").await.unwrap();

        let mut buf = vec![0; 32];

        for _ in 0..20 {
            let (n, peer) = socket.recv_from(&mut buf).await.unwrap();
            log::debug!("b got: {:?}", std::str::from_utf8(&buf[..n]).unwrap());
            assert_eq!(b"Ping Ping Ping", &buf[..n]);
            socket.send_to(b"Pong Pong Pong", peer).await.unwrap();
        }
    });

    let _ = timeoutfut(run_udp((&endpoint).into()), 3).await;
}
