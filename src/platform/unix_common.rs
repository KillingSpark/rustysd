pub unsafe fn setenv(key: &str, value: &str) {
    let k = std::ffi::CString::new(key.as_bytes()).unwrap();
    let v = std::ffi::CString::new(value.as_bytes()).unwrap();

    libc::setenv(k.as_ptr(), v.as_ptr(), 1);
}

use std::os::unix::io::RawFd;
pub unsafe fn unset_cloexec(fd: RawFd) -> Result<(), String> {
    let old_flags = libc::fcntl(fd, libc::F_GETFD, 0);
    if old_flags <= -1 {
        return Err(format!("Couldn't get fd_flags for FD: {}", fd));
    } else {
        // need to actually flip the u32 not just negate the i32.....
        let unset_cloexec_flag = (libc::FD_CLOEXEC as u32 ^ 0xFFFF_FFFF) as i32;
        let new_flags = old_flags & unset_cloexec_flag;

        let result = libc::fcntl(fd, libc::F_SETFD, new_flags);
        if result <= -1 {
            return Err(format!(
                "failed to manually unset the CLOEXEC flag on FD: {}",
                fd
            ));
        }
    }
    Ok(())
}

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
