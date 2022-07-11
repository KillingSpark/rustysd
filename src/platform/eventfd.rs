pub fn notify_event_fds(eventfds: &[EventFd]) {
    for fd in eventfds {
        notify_event_fd(*fd);
    }
}

#[cfg(not(feature = "linux_eventfd"))]
pub use pipe_eventfd::*;

#[cfg(not(feature = "linux_eventfd"))]
mod pipe_eventfd {
    use std::os::unix::io::RawFd;

    use log::{error, trace};

    #[derive(Clone, Copy, Debug)]
    pub struct EventFd(RawFd, RawFd);
    // EventFd(Read,Write)

    impl EventFd {
        pub fn read_end(&self) -> RawFd {
            self.0
        }
        pub fn write_end(&self) -> RawFd {
            self.1
        }
    }

    pub fn make_event_fd() -> Result<EventFd, String> {
        let (r, w) = nix::unistd::pipe().map_err(|e| format!("Error creating pipe, {}", e))?;
        Ok(EventFd(r, w))
    }

    pub fn notify_event_fd(eventfd: EventFd) {
        notify_raw_event_fd(eventfd.1);
    }

    // will be used when service starting outside of the initial starting process is supported
    fn notify_raw_event_fd(eventfd: RawFd) {
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

    pub fn reset_event_fd(eventfd: EventFd) {
        trace!("reset pipe eventfd");
        let buf: *mut [u8] = &mut [0u8; 8][..];
        unsafe {
            let pointer: *mut std::ffi::c_void = buf as *mut std::ffi::c_void;
            libc::read(eventfd.0, pointer, 8)
        };
    }
}

#[cfg(feature = "linux_eventfd")]
pub use linux_eventfd::*;

#[cfg(feature = "linux_eventfd")]
mod linux_eventfd {
    use log::error;
    use log::trace;
    use std::os::unix::io::RawFd;

    #[derive(Clone, Copy)]
    pub struct EventFd(RawFd);

    impl EventFd {
        pub fn read_end(&self) -> RawFd {
            self.0
        }
        pub fn write_end(&self) -> RawFd {
            self.0
        }
    }

    pub fn make_event_fd() -> Result<EventFd, String> {
        let fd = nix::sys::eventfd::eventfd(0, nix::sys::eventfd::EfdFlags::EFD_CLOEXEC)
            .map_err(|e| format!("Error while creating eventfd: {}", e))?;
        Ok(EventFd(fd))
    }

    pub fn notify_event_fd(eventfd: EventFd) {
        notify_raw_event_fd(eventfd.0);
    }

    // will be used when service starting outside of the initial starting process is supported
    fn notify_raw_event_fd(eventfd: RawFd) {
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

    pub fn reset_event_fd(eventfd: EventFd) {
        trace!("reset linux eventfd");
        //something other than 0 so all waiting select() wake up
        let buf: *mut [u8] = &mut [0u8; 8][..];
        unsafe {
            let pointer: *mut std::ffi::c_void = buf as *mut std::ffi::c_void;
            libc::read(eventfd.0, pointer, 8)
        };
        let zeros: *const [u8] = &[0u8; 8][..];
        unsafe {
            let pointer: *const std::ffi::c_void = zeros as *const std::ffi::c_void;
            libc::write(eventfd.0, pointer, 8)
        };
    }
}
