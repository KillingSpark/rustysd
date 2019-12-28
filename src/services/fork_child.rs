use crate::platform::setenv;
use crate::services::Service;
use crate::sockets::Socket;
use crate::units::UnitId;
use std::collections::HashMap;
use std::os::unix::io::RawFd;

fn close_all_unneeded_fds(srvc: &mut Service, sockets: &HashMap<UnitId, &mut Socket>) {
    // This is not really necessary since we mark all fds with FD_CLOEXEC but just to be safe...
    for (id, sock) in sockets.iter() {
        //trace!("[FORK_CHILD {}] CLOSE FDS FOR SOCKET: {}", name, sock.name);
        if !srvc.socket_ids.contains(id) {
            for conf in &sock.sockets {
                match &conf.fd {
                    Some(fd) => {
                        let fd: i32 = (**fd).as_raw_fd();
                        nix::unistd::close(fd).unwrap();
                        //trace!("[FORK_CHILD {}] DO CLOSE FD: {}", name, fd);
                    }
                    None => {
                        //this should not happen but if it does its not too bad
                    }
                }
            }
        }
    }
}

fn setup_env_vars(sockets: &HashMap<UnitId, &mut Socket>, notify_socket_env_var: &str) {
    // The following two lines do deadlock after fork and before exec... I would have loved to just use these
    // This has probably something to do with the global env_lock() that is being used in the std
    // std::env::set_var("LISTEN_FDS", format!("{}", srvc.file_descriptors.len()));
    // std::env::set_var("LISTEN_PID", format!("{}", pid));

    // so lets use some unsafe instead, and use the same libc::setenv that the std uses but we dont care about the lock
    // This is the only thread in this process that is still running so we dont need any lock

    // TODO Maybe it would be better to have a simple wrapper that we can exec with a few sensible args
    // 1. list filedescriptors to keep open (maybe not event that. FD handling can be done here probably?)
    // 2. at least the number of fds
    // 3. the actual executable that should be run + their args
    //
    // This wrapper then does:
    // 1. Maybe close and dup2 fds
    // 2. Set appropriate env variables
    // 3. exec the actual executable we are trying to start here

    // This is all just that complicated because systemd promises to pass the correct PID in the env-var LISTEN_PID...

    let mut num_fds = 0;
    let mut name_lists = Vec::new();

    for sock in sockets.values() {
        //trace!("[FORK_CHILD {}] Counting fds for socket: {}", name, sock_name);
        num_fds += sock.sockets.len();
        name_lists.push(sock.build_name_list());
    }

    let pid = nix::unistd::getpid();
    let pid_str = &format!("{}", pid);
    let fds_str = &format!("{}", num_fds);

    let full_name_list = name_lists.join(":");
    unsafe {
        setenv("LISTEN_FDS", fds_str);
    }
    unsafe {
        setenv("LISTEN_PID", pid_str);
    }
    unsafe {
        setenv("LISTEN_FDNAMES", &full_name_list);
    }
    unsafe {
        setenv("NOTIFY_SOCKET", notify_socket_env_var);
    }

    //trace!(
    //    "[FORK_CHILD {}] pid: {}, ENV: LISTEN_PID: {}  LISTEN_FD: {}, LISTEN_FDNAMES: {}",
    //    name,
    //    pid,
    //    pid_str,
    //    fds_str,
    //    full_name_list
    //);
}

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

fn dup_fds(name: &str, sockets: &HashMap<UnitId, &mut Socket>) {
    // start at 3. 0,1,2 are stdin,stdout,stderr
    let file_desc_offset = 3;
    let mut fd_idx = 0;

    for socket in sockets.values() {
        for sock_conf in &socket.sockets {
            let new_fd = file_desc_offset + fd_idx;
            let old_fd = match &sock_conf.fd {
                Some(fd) => fd.as_raw_fd(),
                None => panic!("No fd found for socket conf"),
            };
            let actual_new_fd = if new_fd as i32 != old_fd {
                //ignore output. newfd might already be closed.
                // TODO check for actual errors other than bad_fd
                let _ = nix::unistd::close(new_fd as i32);
                let actual_new_fd = nix::unistd::dup2(old_fd, new_fd as i32).unwrap();
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
            fd_idx += 1;
        }
    }
}

fn prepare_exec_args(srvc: &Service) -> (std::ffi::CString, Vec<std::ffi::CString>) {
    let split: Vec<&str> = match &srvc.service_config {
        Some(conf) => conf.exec.split(' ').collect(),
        None => unreachable!(),
    };

    let cmd = std::ffi::CString::new(split[0]).unwrap();
    let mut args = Vec::new();

    let exec_name = std::path::PathBuf::from(split[0]);
    let exec_name = exec_name.file_name().unwrap();
    let exec_name: Vec<u8> = exec_name.to_str().unwrap().bytes().collect();
    let exec_name = std::ffi::CString::new(exec_name).unwrap();
    args.push(exec_name);
    for arg in &split[1..] {
        if !arg.is_empty() {
            args.push(std::ffi::CString::new(*arg).unwrap());
        }
    }

    (cmd, args)
}

fn move_into_new_process_group() {
    //make this process the process group leader
    nix::unistd::setpgid(nix::unistd::getpid(), nix::unistd::Pid::from_raw(0)).unwrap();
}

pub fn after_fork_child(
    srvc: &mut Service,
    name: &str,
    sockets: &HashMap<UnitId, &mut Socket>,
    notify_socket_env_var: &str,
    new_stdout: RawFd,
    new_stderr: RawFd,
) {
    // DO NOT USE THE LOGGER HERE. It aquires a global lock which might be held at the time of forking
    // But since this is the only thread that is in the child process the lock will never be released!
    move_into_new_process_group();

    // no more logging after this point!
    // The filedescriptor used by the logger might have been duped to another
    // one and logging into that one would be.... bad
    // Hopefully the close() means that no old logs will get written to that filedescriptor

    close_all_unneeded_fds(srvc, sockets);

    dup_stdio(new_stdout, new_stderr);
    dup_fds(name, sockets);

    setup_env_vars(sockets, notify_socket_env_var);
    let (cmd, args) = prepare_exec_args(srvc);

    eprintln!("EXECV: {:?} {:?}", &cmd, &args);
    match nix::unistd::execv(&cmd, &args) {
        Ok(_) => {
            eprintln!(
                "[FORK_CHILD {}] execv returned Ok()... This should never happen",
                name
            );
        }
        Err(e) => {
            eprintln!("[FORK_CHILD {}] execv errored: {:?}", name, e);
        }
    }
}
