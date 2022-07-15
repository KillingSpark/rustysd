use crate::services::Service;
use crate::units::ServiceConfig;
use std::os::unix::io::RawFd;

fn dup_stdio(new_stdout: RawFd, new_stderr: RawFd) {
    // dup new stdout to fd 1. The other end of the pipe will be read from the service daemon
    let actual_new_fd = nix::unistd::dup2(new_stdout, 1).unwrap();
    if actual_new_fd != 1 {
        panic!(
            "Could not dup the pipe to stdout. Got duped to: {}",
            actual_new_fd
        );
    }
    // dup new stderr to fd 2. The other end of the pipe will be read from the service daemon
    let actual_new_fd = nix::unistd::dup2(new_stderr, 2).unwrap();
    if actual_new_fd != 2 {
        panic!(
            "Could not dup the pipe to stderr. Got duped to: {}",
            actual_new_fd
        );
    }
}

fn dup_fds(name: &str, mut sockets: Vec<RawFd>) -> Result<(), String> {
    // start at 3. 0,1,2 are stdin,stdout,stderr
    let file_desc_offset = 3;
    for fd_idx in 0..sockets.len() {
        let old_fd = sockets[fd_idx];
        let new_fd = (file_desc_offset + fd_idx) as RawFd;

        for fd in sockets.iter_mut().skip(fd_idx) {
            if *fd == new_fd {
                // We need to rescue this fd!
                let rescued_fd =
                    nix::unistd::dup(*fd).map_err(|e| format!("Error while duping fd: {}", e))?;
                let _ = nix::unistd::close(*fd);
                *fd = rescued_fd;
            }
        }

        let actual_new_fd = if new_fd as i32 != old_fd {
            //ignore output. newfd might already be closed.
            // TODO check for actual errors other than bad_fd
            let _ = nix::unistd::close(new_fd as i32);
            let actual_new_fd = nix::unistd::dup2(old_fd, new_fd as i32)
                .map_err(|e| format!("Error while duping fd: {}", e))?;
            let _ = nix::unistd::close(old_fd as i32);
            actual_new_fd
        } else {
            new_fd
        };
        if new_fd != actual_new_fd {
            panic!(
                "Could not dup2 fd {} to {} as required. Was duped to: {}!",
                old_fd, new_fd, actual_new_fd
            );
        }
        unsafe {
            if let Err(msg) = crate::platform::unset_cloexec(new_fd) {
                eprintln!(
                    "[FORK_CHILD {}] Error while unsetting cloexec flag {}",
                    name, msg
                );
            }
        };
    }
    Ok(())
}

fn move_into_new_process_group() {
    //make this process the process group leader
    nix::unistd::setpgid(nix::unistd::getpid(), nix::unistd::Pid::from_raw(0)).unwrap();
}

pub fn after_fork_child(
    srvc: &mut Service,
    conf: &ServiceConfig,
    name: &str,
    socket_fds: Vec<RawFd>,
    new_stdout: RawFd,
    new_stderr: RawFd,
    exec_helper_config: RawFd,
) {
    if let Err(e) = super::fork_os_specific::post_fork_os_specific(srvc) {
        eprintln!("[FORK_CHILD {}] postfork error: {}", name, e);
        std::process::exit(1);
    }

    // DO NOT USE THE LOGGER HERE. It aquires a global lock which might be held at the time of forking
    // But since this is the only thread that is in the child process the lock will never be released!
    move_into_new_process_group();

    // no more logging after this point!
    // The filedescriptor used by the logger might have been duped to another
    // one and logging into that one would be.... bad
    // Hopefully the close() means that no old logs will get written to that filedescriptor

    dup_stdio(new_stdout, new_stderr);

    if let Err(e) = dup_fds(name, socket_fds) {
        eprintln!("[FORK_CHILD {}] error while duping fds: {}", name, e);
        std::process::exit(1);
    }

    if nix::unistd::getuid().is_root() {
        match crate::platform::drop_privileges(
            conf.exec_config.group,
            &conf.exec_config.supplementary_groups,
            conf.exec_config.user,
        ) {
            Ok(()) => { /* Happy */ }
            Err(e) => {
                eprintln!(
                    "[FORK_CHILD {}] could not drop privileges because: {}",
                    name, e
                );
                std::process::exit(1);
            }
        }
    }

    let cmd = std::ffi::CString::new("/proc/self/exe").unwrap();
    let args = &[&std::ffi::CString::new("exec_helper").unwrap()];
    eprintln!("EXECV: {:?} {:?}", &cmd, &args);

    nix::unistd::dup2(exec_helper_config, libc::STDIN_FILENO).unwrap();
    nix::unistd::close(exec_helper_config).unwrap();
    match nix::unistd::execv(&cmd, args) {
        Ok(_) => {
            eprintln!(
                "[FORK_CHILD {}] execv returned Ok()... This should never happen",
                name
            );
        }
        Err(e) => {
            eprintln!("[FORK_CHILD {}] execv errored: {:?}", name, e);
            std::process::exit(1);
        }
    }
}
