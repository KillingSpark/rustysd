use std::io::Write;
use std::os::unix::net::UnixDatagram;
use std::os::unix::net::UnixStream;

fn main() {
    loop {
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
    }
}
