use std::env;
use std::os::unix::net::{UnixListener, UnixStream};
use std::os::unix::io::{FromRawFd};
use std::io::Read;

extern crate nix;

fn handle_client(mut stream: UnixStream) {
    println!("Got new stream! Now printing stuff from the stream:");
    let mut data = [0u8;512];
    loop {
        match stream.read(&mut data[..]) {
            Ok(bytes) => print!("{}", String::from_utf8(data[0..bytes].to_vec()).unwrap()),
            Err(e) => println!("\n Got error from stream: {}", e),
        }
    }
}

fn main() {
    let pid_should: i32 = String::from_utf8(env::var("LISTEN_PID").unwrap().as_bytes().to_vec()).unwrap().parse().unwrap();
    let pid_is = nix::unistd::getpid();

    assert_eq!(pid_should, pid_is);

    
    let num_fds: u32 = String::from_utf8(env::var("LISTEN_FDS").unwrap().as_bytes().to_vec()).unwrap().parse().unwrap();
    assert!(num_fds >= 1);
    let unix_listen: UnixListener = unsafe{UnixListener::from_raw_fd(3)};

    println!("STARTED DEAMON WITH PID: {} AND FDS: {}", env::var("LISTEN_FDS").unwrap(), env::var("LISTEN_PID").unwrap());

    for stream in unix_listen.incoming() {
        match stream {
            Ok(stream) => {
                /* connection succeeded */
                std::thread::spawn(|| handle_client(stream));
            }
            Err(err) => {
                /* connection failed */
                println!("Error while accepting new connections: {}", err);
                break;
            }
        }
    }
}