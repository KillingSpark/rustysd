use std::os::unix::io::AsRawFd;
use std::os::unix::net::SocketAddr;
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;

pub fn select<Key: Clone>(
    listeners: &Vec<(Key, &UnixListener)>,
    time_out: Option<&mut nix::sys::time::TimeVal>,
) -> nix::Result<Vec<(Key, (UnixStream, SocketAddr))>> {
    let mut fd_to_name = Vec::new();
    let mut streams = Vec::new();

    let mut fd_set = nix::sys::select::FdSet::new();
    let mut fd_set_err = nix::sys::select::FdSet::new();
    let mut max_fd = 0;
    for (name, listener) in listeners {
        let fd = listener.as_raw_fd();
        fd_set.insert(fd);
        fd_set_err.insert(fd);
        fd_to_name.push((fd, (name, listener)));
        if max_fd < fd {
            max_fd = fd;
        }
    }

    loop {
        match nix::sys::select::select(
            max_fd + 1,
            Some(&mut fd_set),
            None,
            Some(&mut fd_set_err),
            time_out,
        ) {
            Ok(_) => break,
            Err(e) => match e {
                nix::Error::Sys(nix::errno::EINTR) => break,
                _ => return Err(e),
            },
        }
    }

    for (fd, (name, listener)) in fd_to_name {
        if fd_set.contains(fd) {
            streams.push((name.clone(), listener.accept().unwrap()));
        }
    }

    Ok(streams)
}
