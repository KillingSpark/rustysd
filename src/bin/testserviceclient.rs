use std::io::Write;
use std::os::unix::net::UnixDatagram;
use std::os::unix::net::UnixStream;

extern crate nix;

fn main() {
    match unsafe { nix::unistd::fork() } {
        Ok(nix::unistd::ForkResult::Parent { .. }) => loop {
            let mut stream = UnixStream::connect("./sockets/servicelog_stream").unwrap();
            stream.write_all(b"AAAA\n").unwrap();
            stream.write_all(b"BBBB\n").unwrap();
            stream.write_all(b"CCCC\n").unwrap();
            stream.write_all(b"This is a weird text protocol").unwrap();
            std::mem::drop(stream);

            let socket_path = std::env::var("NOTIFY_SOCKET").unwrap();
            let stream = UnixDatagram::unbound().unwrap();
            stream.connect(socket_path).unwrap();
            stream
                .send(&b"STATUS=Sent a message to the server\n"[..])
                .unwrap();

            std::thread::sleep(std::time::Duration::from_secs(2));
        },
        Ok(nix::unistd::ForkResult::Child) => loop {
            let mut stream = UnixStream::connect("./sockets/servicelog_alt_stream").unwrap();
            stream.write_all(b"DDDD\n").unwrap();
            stream.write_all(b"EEEE\n").unwrap();
            stream.write_all(b"FFFF\n").unwrap();
            stream
                .write_all(b"This is a weird text protocol on the alternative socket")
                .unwrap();
            std::mem::drop(stream);

            let socket_path = std::env::var("NOTIFY_SOCKET").unwrap();
            let stream = UnixDatagram::unbound().unwrap();
            stream.connect(socket_path).unwrap();
            stream
                .send(&b"STATUS=Sent a message to the server\n"[..])
                .unwrap();

            std::thread::sleep(std::time::Duration::from_secs(2));
        },
        Err(e) => eprintln!("Error while forking: {}", e),
    }
}
