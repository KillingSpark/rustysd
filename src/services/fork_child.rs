use std::os::unix::io::RawFd;

/// After forking we setup the all filedescriptors, move into a new process group and then exec the exec_helper
///
/// Note that this is called between fork and exec. This means this needs to be careful about what we call here!
/// At least on linux this is a good reference: https://man7.org/linux/man-pages/man7/signal-safety.7.html
pub fn after_fork_child(
    selfpath: &std::ffi::CStr,
    self_args: &[*const libc::c_char],
    socket_fds: &mut [RawFd],
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

    // Now we may at least write to stderr
    write_to_stderr("Prepare fork child before execing!");

    // Lets move into a new process group before execing
    move_into_new_process_group();

    // Dup all the fds for the service here, because we use SO_CLOEXEC on all fds so doing it after exec isn't possible
    dup_fds(socket_fds);

    // Just so we have a clearer picture on what is happening while debugging
    write_to_stderr("Exec the exec helper");

    // Finally exec the exec_helper
    match unsafe { libc::execv(selfpath.as_ptr(), self_args.as_ptr().cast()) } {
        -1 => {
            write_to_stderr("execv errored");
            std::process::exit(1);
        }
        _ => {
            write_to_stderr("execv returned Ok()... This should never happen");
            std::process::exit(1);
        }
    }
}

fn write_to_stderr(msg: &str) {
    unsafe {
        libc::write(
            libc::STDERR_FILENO,
            (msg.as_bytes() as *const [u8]).cast(),
            msg.as_bytes().len() as _,
        );
        libc::write(libc::STDERR_FILENO, (&b'\n' as *const u8).cast(), 1 as _);
    }
}

fn dup_stdio(new_stdout: RawFd, new_stderr: RawFd, exec_helper_config: RawFd) {
    fn dup_one_stdio(
        old_stdio: RawFd,
        new_stdio: RawFd,
        fd_name: &str,
        write_error_msg_to_stderr: bool,
    ) {
        let actual_new_fd = unsafe { libc::dup2(old_stdio, new_stdio) };
        if actual_new_fd != new_stdio {
            if write_error_msg_to_stderr {
                let msg = "Could not dup fd";
                write_to_stderr(msg);
                write_to_stderr(fd_name);
            }
            std::process::exit(1);
        }
        unsafe { libc::close(old_stdio) };
    }

    // First dup stderr so we can potentially log other dup errors
    dup_one_stdio(new_stderr, libc::STDERR_FILENO, "stderr", false);
    dup_one_stdio(new_stdout, libc::STDOUT_FILENO, "stdout", true);
    dup_one_stdio(exec_helper_config, libc::STDIN_FILENO, "stdin", true);
}

fn move_into_new_process_group() {
    //make this process the process group leader
    unsafe {
        if libc::setpgid(libc::getpid(), 0) != 0 {
            write_to_stderr("Could not move to new process group");
            std::process::exit(1);
        }
    };
}

fn dup_fds(sockets: &mut [RawFd]) {
    // start at 3. 0,1,2 are stdin,stdout,stderr
    let file_desc_offset = (libc::STDERR_FILENO + 1) as usize;
    for fd_idx in 0..sockets.len() {
        let old_fd = sockets[fd_idx];
        let new_fd = (file_desc_offset + fd_idx) as RawFd;

        for fd in sockets.iter_mut().skip(fd_idx) {
            if *fd == new_fd {
                // We need to rescue this fd!
                let rescued_fd = unsafe { libc::dup(*fd) };
                if rescued_fd < 0 {
                    write_to_stderr("Could not dup fd");
                    std::process::exit(1);
                }
                let _ = unsafe { libc::close(*fd) };
                *fd = rescued_fd;
            }
        }

        if new_fd as i32 != old_fd {
            //ignore output. newfd might already be closed.
            // TODO check for actual errors other than bad_fd
            let _ = nix::unistd::close(new_fd as i32);
            let actual_new_fd = unsafe { libc::dup2(old_fd, new_fd as RawFd) };
            if actual_new_fd != new_fd {
                write_to_stderr("Could not dup2 fd");
                std::process::exit(1);
            }
            let _ = unsafe { libc::close(old_fd as i32) };
        } else {
            // nothing to do, already correct fd
        }

        unsafe {
            unset_cloexec(new_fd);
        }
    }
}

unsafe fn unset_cloexec(fd: RawFd) {
    let old_flags = libc::fcntl(fd, libc::F_GETFD, 0);
    if old_flags <= -1 {
        write_to_stderr("Couldn't get fd_flags for FD");
        std::process::exit(1);
    } else {
        // need to actually flip the u32 not just negate the i32.....
        let unset_cloexec_flag = (libc::FD_CLOEXEC as u32 ^ 0xFFFF_FFFF) as i32;
        let new_flags = old_flags & unset_cloexec_flag;

        let result = libc::fcntl(fd, libc::F_SETFD, new_flags);
        if result <= -1 {
            write_to_stderr("failed to manually unset the CLOEXEC flag on FD");
            std::process::exit(1);
        }
    }
}
