use std::io::prelude::*;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const BUFFER_SIZE: usize = 1024 * 2;

pub fn start(client_socket: SocketAddr, remote_socket: SocketAddr) {
    let listener = TcpListener::bind(client_socket).unwrap_or_else(|err| {
        println!("Failed to start listener.");
        panic!(err);
    });

    for stream in listener.incoming() {
        match stream {
            Err(e) => println!("Error: {}", e),
            Ok(stream) => {
                thread::spawn(move || handle_client(stream, remote_socket));
            }
        }
    }
}

fn handle_client(mut stream: TcpStream, remote_socket: SocketAddr) {
    let (forward_tx, forward_rx) = mpsc::channel();
    let (backward_tx, backward_rx) = mpsc::channel();

    let mut cl_stream = stream;
    let mut lc_stream = cl_stream
        .try_clone()
        .expect("Failed to clone client stream");

    let mut lr_stream =
        TcpStream::connect(remote_socket).expect("Failed to connect to remote host");
    let mut rl_stream = lr_stream
        .try_clone()
        .expect("Failed to clone remote stream");

    thread::spawn(move || {
        // client -> local
        let mut data = [0 as u8; BUFFER_SIZE]; // using 2048 byte buffer
        loop {
            match cl_stream.read(&mut data) {
                Ok(size) => {
                    if let Err(_) = forward_tx.send((data, size)) {
                        break;
                    }
                    if size == 0 {
                        break;
                    }
                }
                Err(e) => {
                    forward_tx.send((data, 0)).unwrap();
                    break;
                }
            }
        }
        cl_stream.shutdown(Shutdown::Read);
    });

    thread::spawn(move || {
        // local -> remote
        loop {
            if let Ok((mut data, size)) = forward_rx.recv() {
                if size == 0 {
                    break;
                }
                if let Err(_) = lr_stream.write(&mut data[..size]) {
                    break;
                }
            } else {
                break;
            }
        }
        lr_stream.shutdown(Shutdown::Write);
        return;
    });

    thread::spawn(move || {
        // remote -> local
        let mut data = [0 as u8; BUFFER_SIZE];
        loop {
            match rl_stream.read(&mut data) {
                Ok(size) => {
                    if let Err(_) = backward_tx.send((data, size)) {
                        break;
                    }
                    if size == 0 {
                        break;
                    }
                }
                Err(e) => {
                    backward_tx.send((data, 0)).unwrap();
                    break;
                }
            }
        }
        rl_stream.shutdown(Shutdown::Read);
    });

    // local -> client
    loop {
        if let Ok((mut data, size)) = backward_rx.recv() {
            if size == 0 {
                break;
            }
            if let Err(_) = lc_stream.write(&mut data[..size]) {
                break;
            }
        } else {
            break;
        }
    }
    lc_stream.shutdown(Shutdown::Write);
    return;
}
