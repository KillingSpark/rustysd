use std::os::unix::io::RawFd;

pub fn make_seqpacket_socket(path: &std::path::PathBuf) -> Result<RawFd, String> {
    //let addr_family = nix::sys::socket::AddressFamily::Unix;
    //let sock_type = nix::sys::socket::SockType::SeqPacket;
    //let flags = nix::sys::socket::SockFlag::empty(); //flags can be set by using the fnctl calls later if necessary
    let protocol = 0; // not really important, used to choose protocol but we dont support sockets where thats relevant

    let unix_addr = nix::sys::socket::UnixAddr::new(path).unwrap();

    let fd = unsafe { libc::socket(libc::AF_UNIX, libc::SOCK_SEQPACKET, protocol) };
    if fd < 0 {
        return Err(format!(
            "Could not opensequential packet  socket. Result was: {}",
            fd,
        ));
    }
    // then bind the socket to the path
    nix::sys::socket::bind(fd, &unix_addr).unwrap();
    // then make the socket an accepting one
    nix::sys::socket::listen(fd, 128).unwrap();

    Ok(fd)
}
