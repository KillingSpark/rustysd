use std::io::Write;
use std::os::unix::net::UnixStream;
use std::os::unix::net::UnixDatagram;

fn main() {
    loop {
        let mut stream = UnixStream::connect("./sockets/servicelog_stream").unwrap();
        stream.write_all("AAAA\n".as_bytes()).unwrap();
        stream.write_all("BBBB\n".as_bytes()).unwrap();
        stream.write_all("CCCC\n".as_bytes()).unwrap();
        stream
            .write_all("This is a weird text protocol".as_bytes())
            .unwrap();
        std::mem::drop(stream);

        let socket_path = std::env::var("NOTIFY_SOCKET").unwrap();
        let stream = UnixDatagram::unbound().unwrap();
        stream.connect(socket_path).unwrap();
        stream.send(&b"STATUS=Sent a message to the server\n"[..]).unwrap();

        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}
