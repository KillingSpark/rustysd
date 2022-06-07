#![allow(dead_code)]

use std::env;
use std::io::Read;
use std::os::unix::io::FromRawFd;
use std::os::unix::net::{UnixDatagram, UnixListener, UnixStream};

extern crate nix;

// send stuff to this service with:
// echo "REEE" | socat - TCP-CONNECT:127.0.0.1:8080
// echo "REEE" | socat - UDP-CONNECT:127.0.0.1:8081
// echo "REEE" | socat - UNIX-CONNECT:./servicelog_stream
// echo "REEE" | socat - UNIX-SENDTO:./servicelog_datagram

fn handle_unix_client(mut stream: UnixStream) {
    println!("Got new unix stream! Now printing stuff from the stream:");
    let mut data = [0u8; 512];
    loop {
        match stream.read(&mut data[..]) {
            Ok(bytes) => {
                if bytes == 0 {
                    println!("\nUnix stream finished");
                    break;
                } else {
                    print!("{}", String::from_utf8(data[0..bytes].to_vec()).unwrap())
                }
            }
            Err(e) => println!("\n Got error from unix stream: {}", e),
        }
    }
}

use std::net::UdpSocket;
fn handle_upd(fd: i32) {
    std::thread::spawn(move || {
        let stream: UdpSocket = unsafe { UdpSocket::from_raw_fd(fd) };
        let mut data = [0u8; 512];
        loop {
            match stream.recv(&mut data[..]) {
                Ok(bytes) => {
                    print!("Got new bytes on udp socket! Now printing stuff from the stream: ");
                    print!("{}", String::from_utf8(data[0..bytes].to_vec()).unwrap())
                }
                Err(e) => {
                    println!("\n Got error from udp socket: {}", e);
                    return;
                }
            }
        }
    });
}

fn handle_unix_datagram(fd: i32) {
    std::thread::spawn(move || {
        let stream = unsafe { UnixDatagram::from_raw_fd(fd) };
        let mut data = [0u8; 512];
        loop {
            match stream.recv(&mut data[..]) {
                Ok(bytes) => {
                    print!("Got new bytes on unix datagram socket! Now printing stuff from the stream: ");
                    print!("{}", String::from_utf8(data[0..bytes].to_vec()).unwrap())
                }
                Err(e) => {
                    println!("\n Got error from unix datagram socket: {}", e);
                    return;
                }
            }
        }
    });
}

fn unix_accept(fd: i32) {
    std::thread::spawn(move || {
        let unix_listen: UnixListener = unsafe { UnixListener::from_raw_fd(fd) };
        for stream in unix_listen.incoming() {
            match stream {
                Ok(stream) => {
                    /* connection succeeded */
                    std::thread::spawn(|| handle_unix_client(stream));
                }
                Err(err) => {
                    /* connection failed */
                    println!("Error while accepting new unix connections: {}", err);
                    break;
                }
            }
        }
    });
}

//fn handle_unix_seq_pack(fd: i32) {
//    println!("Got new unix seqpack stream! Now printing stuff from the stream:");
//    let mut buf = [0u8; 512];
//    loop {
//        let bytes = match nix::unistd::read(fd, &mut buf[..]) {
//            Ok(b) => b,
//            Err(e) => {
//                println!("Error while reading seqpack stream: {}", e);
//                return;
//            }
//        };
//        print!("{:?}", &buf[0..bytes]);
//    }
//}
//
//fn unix_seq_pack_accept(fd: i32) {
//    std::thread::spawn(move || {
//        loop {
//            let mut new_con_sock_addr = libc::sockaddr {
//                sa_data: [0i8; 14],
//                sa_family: libc::AF_UNIX as u16,
//            };
//            let mut addr_len = 0;
//            let new_con_fd =
//                unsafe { libc::accept(fd, &mut new_con_sock_addr, &mut addr_len) };
//            if new_con_fd < 0 {
//                println!("Error while accepting unix seqpack fd: {}", new_con_fd);
//                return;
//            }
//            handle_unix_seq_pack(new_con_fd);
//        }
//    });
//}

use std::net::TcpListener;
use std::net::TcpStream;
fn handle_tcp_client(mut stream: TcpStream) {
    println!("Got new tcp stream! Now printing stuff from the stream:");
    let mut data = [0u8; 512];
    loop {
        match stream.read(&mut data[..]) {
            Ok(bytes) => print!("{}", String::from_utf8(data[0..bytes].to_vec()).unwrap()),
            Err(e) => println!("\n Got error from tcp stream: {}", e),
        }
    }
}
fn tcp_accept(fd: i32) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let listen = unsafe { TcpListener::from_raw_fd(fd) };
        for stream in listen.incoming() {
            match stream {
                Ok(stream) => {
                    /* connection succeeded */
                    std::thread::spawn(|| handle_tcp_client(stream));
                }
                Err(err) => {
                    /* connection failed */
                    println!("Error while accepting new tcp connections: {}", err);
                    break;
                }
            }
        }
    })
}

fn main() {
    if (nix::unistd::getpid().as_raw() / 10) % 10 != 0 {
        panic!("My service is very bad. It immediately panics at startup.");
    }

    println!(
        "STARTED DEAMON WITH PID: {} AND FDS: {}",
        env::var("LISTEN_PID").unwrap(),
        env::var("LISTEN_FDS").unwrap(),
    );

    eprintln!("Test stderr print!");

    println!("Args: {:?}", std::env::args().collect::<Vec<_>>());

    let pid_should: i32 = String::from_utf8(env::var("LISTEN_PID").unwrap().as_bytes().to_vec())
        .unwrap()
        .parse()
        .unwrap();
    let pid_is = nix::unistd::getpid();

    assert_eq!(pid_should, pid_is.as_raw());

    let num_fds: u32 = String::from_utf8(env::var("LISTEN_FDS").unwrap().as_bytes().to_vec())
        .unwrap()
        .parse()
        .unwrap();
    assert!(num_fds >= 1);

    unix_accept(3);
    unix_accept(6);
    //tcp_accept(4);
    //tcp_accept(9);
    //handle_upd(5);
    //handle_upd(10);
    handle_unix_datagram(4);
    handle_unix_datagram(7);
    //unix_seq_pack_accept(5);
    //unix_seq_pack_accept(8);

    // act as if there was a lot of time used for setting up the service
    //std::thread::sleep(std::time::Duration::from_secs(3));

    // send the READY=1 message amongst some other stuff
    let socket_path = std::env::var("NOTIFY_SOCKET").unwrap();
    let stream = UnixDatagram::unbound().unwrap();
    stream.connect(socket_path).unwrap();
    stream.send(&b"STATUS=Next message that should be read before the READY message\nREADY=1\nSTATUS=Next message that should not be read directly after the fork\n"[..]).unwrap();

    //create a child so we can see that orphanes are killed too
    match unsafe { nix::unistd::fork() } {
        Ok(nix::unistd::ForkResult::Child) => {
            std::thread::sleep(std::time::Duration::from_secs(1000000));
        }
        _ => {}
    }
    // random service failure because we write horrible services that crash constantly
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(10));
        std::process::exit(1);
    });

    let mut counter = 0;
    loop {
        stream
            .send(format!("STATUS=Looping since {} seconds\n", counter).as_bytes())
            .unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        counter += 1;
    }
}
