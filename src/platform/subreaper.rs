#[cfg(target_os = "linux")]
pub fn become_subreaper(set: bool) {
    use log::error;

    unsafe {
        // Set subreaper to collect all zombies left behind by the services
        let res = if set {
            libc::prctl(libc::PR_SET_CHILD_SUBREAPER, 1)
        } else {
            libc::prctl(libc::PR_SET_CHILD_SUBREAPER, 0)
        };
        if res < 0 {
            error!("Couldnt set subreaper for rustysd");
            return;
        }
    }
}
#[cfg(any(
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly"
))]
extern "C" {
    fn procctl(
        idtype: libc::c_int,
        id: libc::c_int,
        cmd: libc::c_int,
        args: *const std::ffi::c_void,
    ) -> libc::c_int;
}

#[cfg(any(
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly"
))]
const PROC_REAP_ACQUIRE: libc::c_int = 2;
#[cfg(any(
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly"
))]
const PROC_REAP_RELEASE: libc::c_int = 3;

#[cfg(any(
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly"
))]
pub fn become_subreaper(set: bool) {
    unsafe {
        // Set subreaper to collect all zombies left behind by the services
        // TODO make pull request to libc to include this

        let res = if set {
            procctl(
                libc::P_PID as i32,
                libc::getpid(),
                PROC_REAP_ACQUIRE,
                std::ptr::null(),
            )
        } else {
            procctl(
                libc::P_PID as i32,
                libc::getpid(),
                PROC_REAP_RELEASE,
                std::ptr::null(),
            )
        };
        if res < 0 {
            eprintln!("Couldnt set subreaper for rustysd");
            return;
        } else {
            eprintln!("Acquire/Release subreaper privilege successfully");
        }
    }
}
