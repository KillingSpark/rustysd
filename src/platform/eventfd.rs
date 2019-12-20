use std::os::unix::io::RawFd;

#[cfg(feature = "linux_eventfd")]
pub fn make_event_fd() -> Result<(RawFd, RawFd), String> {
    return nix::sys::eventfd::eventfd(0, nix::sys::eventfd::EfdFlags::EFD_CLOEXEC)
        .map_err(|e| format!("Error while creating eventfd: {}", e))
        .map(|fd| (fd, fd));
}

#[cfg(not(feature = "linux_eventfd"))]
pub fn make_event_fd() -> Result<(RawFd, RawFd), String> {
    return nix::unistd::pipe().map_err(|e| format!("Error creating pipe, {}", e));
}

// will be used when service starting outside of the initial starting process is supported
pub fn notify_event_fd(eventfd: RawFd) {
    //something other than 0 so all waiting select() wake up
    let zeros: *const [u8] = &[1u8; 8][..];

    unsafe {
        let pointer: *const std::ffi::c_void = zeros as *const std::ffi::c_void;
        let x = libc::write(eventfd, pointer, 8);
        if x <= 0 {
            error!("Did not notify eventfd {}: err: {}", eventfd, x);
        } else {
            trace!("notify eventfd");
        }
    };
}

#[cfg(not(feature = "linux_eventfd"))]
pub fn reset_event_fd(eventfd: RawFd) {
    trace!("reset pipe eventfd");
    let buf: *mut [u8] = &mut [0u8; 8][..];
    unsafe {
        let pointer: *mut std::ffi::c_void = buf as *mut std::ffi::c_void;
        libc::read(eventfd, pointer, 8)
    };
}

#[cfg(feature = "linux_eventfd")]
pub fn reset_event_fd(eventfd: RawFd) {
    trace!("reset linux eventfd");
    //something other than 0 so all waiting select() wake up
    let buf: *mut [u8] = &mut [0u8; 8][..];
    unsafe {
        let pointer: *mut std::ffi::c_void = buf as *mut std::ffi::c_void;
        libc::read(eventfd, pointer, 8)
    };
    let zeros: *const [u8] = &[0u8; 8][..];
    unsafe {
        let pointer: *const std::ffi::c_void = zeros as *const std::ffi::c_void;
        libc::write(eventfd, pointer, 8)
    };
}

pub fn notify_event_fds(eventfds: &[RawFd]) {
    for fd in eventfds {
        notify_event_fd(*fd);
    }
}
