use std::os::unix::io::RawFd;

fn dup_stdio(new_stdout: RawFd, new_stderr: RawFd, exec_helper_config: RawFd) {
    // dup new stdout to fd 1. The other end of the pipe will be read from the service daemon
    let actual_new_fd = nix::unistd::dup2(new_stdout, libc::STDOUT_FILENO).unwrap();
    if actual_new_fd != 1 {
        panic!(
            "Could not dup the pipe to stdout. Got duped to: {}",
            actual_new_fd
        );
    }
    // dup new stderr to fd 2. The other end of the pipe will be read from the service daemon
    let actual_new_fd = nix::unistd::dup2(new_stderr, libc::STDERR_FILENO).unwrap();
    if actual_new_fd != 2 {
        panic!(
            "Could not dup the pipe to stderr. Got duped to: {}",
            actual_new_fd
        );
    }

    nix::unistd::dup2(exec_helper_config, libc::STDIN_FILENO).unwrap();
    nix::unistd::close(exec_helper_config).unwrap();
}

fn dup_fds(name: &str, mut sockets: Vec<RawFd>) -> Result<(), String> {
    // start at 3. 0,1,2 are stdin,stdout,stderr
    let file_desc_offset = (libc::STDERR_FILENO + 1) as usize;
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
    selfpath: &std::ffi::CStr,
    self_args: &[&std::ffi::CStr],
    name: &str,
    socket_fds: Vec<RawFd>,
    new_stdout: RawFd,
    new_stderr: RawFd,
    exec_helper_config: RawFd,
) {
    // DO NOT USE THE LOGGER HERE. It aquires a global lock which might be held at the time of forking
    // But since this is the only thread that is in the child process the lock will never be released!
    //
    // Also:
    // The filedescriptor used by the logger might have been duped to another
    // one and logging into that one would be.... bad
    // Hopefully the close() means that no old logs will get written to that filedescriptor

    // Setup the new stdio so println! and eprintln! go to the expected fds
    dup_stdio(new_stdout, new_stderr, exec_helper_config);

    // Lets move into a new process group before execing
    move_into_new_process_group();

    // Dup all the fds for the service here, because we use SO_CLOEXEC on all fds so doing it after exec isn't possible
    if let Err(e) = dup_fds(name, socket_fds) {
        eprintln!("[FORK_CHILD {}] error while duping fds: {}", name, e);
        std::process::exit(1);
    }

    // Just so we have a clearer picture on what is happening while debugging
    eprintln!("EXECV: {:?} {:?}", &selfpath, self_args);

    // Finally exec the exec_helper
    match nix::unistd::execv(selfpath, self_args) {
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
