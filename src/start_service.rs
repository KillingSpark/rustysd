use crate::services::{Service, ServiceStatus};
use crate::units::*;
use std::os::unix::io::AsRawFd;
use std::os::unix::io::RawFd;
use std::os::unix::net::UnixDatagram;

use std::sync::{Arc, Mutex};

fn after_fork_child(
    srvc: &mut Service,
    name: &str,
    sockets: &SocketTable,
    notify_socket_env_var: &str,
    new_stdout: RawFd,
    new_stderr: RawFd,
) {
    // DO NOT USE THE LOGGER HERE. It aquires a global lock which might be held at the time of forking
    // But since this is the only thread that is in the child process the lock will never be released!

    //here we are in the child process. We need to close every file descriptor we dont need anymore after the exec

    // TODO maybe all fd's should be marked with FD_CLOEXEC when openend
    // and here we only unflag those that we want to keep?
    //trace!("[FORK_CHILD {}] CLOSING FDS", name);

    let pid = nix::unistd::getpid();
    for sock_unit in sockets.values() {
        if let UnitSpecialized::Socket(sock) = &sock_unit.specialized {
            if !srvc.socket_names.contains(&sock.name) {
                //trace!("[FORK_CHILD {}] CLOSE FDS FOR SOCKET: {}", name, sock.name);
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
            } else {
                //trace!("[FORK_CHILD {}] DONT CLOSE FDS", name);
            }
        }
    }

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

    let sockets_by_name = get_sockets_by_name(sockets);
    for sock_name in &srvc.socket_names {
        //trace!("[FORK_CHILD {}] Counting fds for socket: {}", name, sock_name);
        match sockets_by_name.get(sock_name) {
            Some(sock) => {
                num_fds += sock.sockets.len();
                name_lists.push(sock.build_name_list());
            }
            None => eprintln!(
                "[FORK_CHILD {}] Socket was specified that cannot be found: {}",
                name, sock_name
            ),
        }
    }

    let pid_str = &format!("{}", pid);
    let fds_str = &format!("{}", num_fds);

    unsafe fn setenv(key: &str, value: &str) {
        let k = std::ffi::CString::new(key.as_bytes()).unwrap();
        let v = std::ffi::CString::new(value.as_bytes()).unwrap();

        libc::setenv(k.as_ptr(), v.as_ptr(), 1);
    }
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

    // no more logging after this point!
    // The filedescriptor used by the logger might have been duped to another
    // one and logging into that one would be.... bad
    // Hopefully the close() means that no old logs will get written to that filedescriptor

    // dup new stdout to fd 1. The other end of the pipe will be read from the service daemon
    let actual_new_fd = nix::unistd::dup2(new_stdout, 1).unwrap();
    if actual_new_fd != 1 {
        panic!("Could not dup the pipe to stdout. Got duped to: {}", actual_new_fd);
    }
    // dup new stderr to fd 2. The other end of the pipe will be read from the service daemon
    let actual_new_fd = nix::unistd::dup2(new_stderr, 2).unwrap();
    if actual_new_fd != 2 {
        panic!("Could not dup the pipe to stderr. Got duped to: {}", actual_new_fd);
    }

    // start at 3. 0,1,2 are stdin,stdout,stderr
    let file_desc_offset = 3;
    let mut fd_idx = 0;

    for sock_name in &srvc.socket_names {
        match sockets_by_name.get(sock_name) {
            Some(socket) => {
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
                    // unset the CLOEXEC flags on the relevant FDs
                    let old_flags = unsafe { libc::fcntl(new_fd, libc::F_GETFD, 0) };
                    if old_flags <= -1 {
                        eprintln!(
                            "[FORK_CHILD {}] failed to manually get the FD flag on fd: {}",
                            name, new_fd
                        )
                    } else {
                        // need to actually flip the u32 not just negate the i32.....
                        let unset_cloexec_flag = (libc::FD_CLOEXEC as u32 ^ 0xFFFFFFFF) as i32;
                        let new_flags = old_flags & unset_cloexec_flag;

                        let result = unsafe { libc::fcntl(new_fd, libc::F_SETFD, new_flags) };
                        if result <= -1 {
                            eprintln!(
                                    "[FORK_CHILD {}] failed to manually unset the CLOEXEC flag on fd: {}", name, new_fd
                                )
                        }
                    }
                    fd_idx += 1;
                }
            }
            None => eprintln!(
                "[FORK_CHILD {}] Socket was specified that cannot be found: {}",
                name, sock_name
            ),
        }
    }

    let split: Vec<&str> = match &srvc.service_config {
        Some(conf) => conf.exec.split(' ').collect(),
        None => unreachable!(),
    };

    let cmd = std::ffi::CString::new(split[0]).unwrap();
    let mut args = Vec::new();
    
    let exec_name = std::path::PathBuf::from(split[0]);
    let exec_name = exec_name.file_name().unwrap();
    let exec_name: Vec<u8>= exec_name.to_str().unwrap().bytes().collect();
    let exec_name = std::ffi::CString::new(exec_name).unwrap();
    args.push(exec_name);
    for arg in &split[1..] {
        if arg.len() > 0 {
            args.push(std::ffi::CString::new(*arg).unwrap());
        }
    }

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

fn after_fork_parent(
    srvc: &mut Service,
    name: String,
    new_pid: nix::unistd::Pid,
    notify_socket_env_var: &std::path::Path,
    stream: &UnixDatagram,
) {
    srvc.pid = Some(new_pid);

    trace!(
        "[FORK_PARENT] Service: {} forked with pid: {}",
        name,
        srvc.pid.unwrap()
    );

    if let Some(conf) = &srvc.service_config {
        if let ServiceType::Notify = conf.srcv_type {
            trace!(
                "[FORK_PARENT] Waiting for a notification on: {:?}",
                &notify_socket_env_var
            );

            let mut buf = [0u8; 512];
            loop {
                let bytes = stream.recv(&mut buf[..]).unwrap();
                srvc.notifications_buffer
                    .push_str(&String::from_utf8(buf[..bytes].to_vec()).unwrap());
                crate::notification_handler::handle_notifications_from_buffer(srvc, &name);
                if let ServiceStatus::Running = srvc.status {
                    break;
                } else {
                    trace!("[FORK_PARENT] Service still not ready",);
                }
            }
        } else {
            trace!("[FORK_PARENT] service {} doesnt notify", name);
            srvc.status = ServiceStatus::Running;
        }
    }
}

fn start_service_with_filedescriptors(
    srvc: &mut Service,
    name: String,
    sockets: ArcMutSocketTable,
    notification_socket_path: std::path::PathBuf,
) {
    // check if executable even exists
    let split: Vec<&str> = match &srvc.service_config {
        Some(conf) => conf.exec.split(' ').collect(),
        None => unreachable!(),
    };

    let cmd = std::path::PathBuf::from(split[0]);
    if !cmd.exists() {
        error!(
            "The service {} specified an executable that does not exist: {:?}",
            name, &cmd
        );
        srvc.status = ServiceStatus::Stopped;
        return;
    }
    if !cmd.is_file() {
        error!(
            "The service {} specified an executable that is not a file: {:?}",
            name, &cmd
        );
        srvc.status = ServiceStatus::Stopped;
        return;
    }

    // 1. fork
    // 2. in fork use dup2 to map all relevant file desrciptors to 3..x
    // 3. in fork mark all other file descriptors with FD_CLOEXEC
    // 4. set relevant env varibales $LISTEN_FDS $LISTEN_PID
    // 4. execve the cmd with the args

    // setup socket for notifications from the service
    if !notification_socket_path.exists() {
        std::fs::create_dir_all(&notification_socket_path).unwrap();
    }
    let daemon_socket_path = notification_socket_path.join(format!("{}.notifiy_socket", &name));

    // NOTIFY_SOCKET
    let notify_socket_env_var = if daemon_socket_path.starts_with(".") {
        let cur_dir = std::env::current_dir().unwrap();
        cur_dir.join(&daemon_socket_path)
    } else {
        daemon_socket_path
    };

    let stream = {
        if let Some(stream) = &srvc.notifications {
            stream.clone()
        } else {
            if notify_socket_env_var.exists() {
                std::fs::remove_file(&notify_socket_env_var).unwrap();
            }
            let stream = UnixDatagram::bind(&notify_socket_env_var).unwrap();
            // close these fd's on exec. They must not show up in child processes
            let new_listener_fd = stream.as_raw_fd();
            nix::fcntl::fcntl(
                new_listener_fd,
                nix::fcntl::FcntlArg::F_SETFD(nix::fcntl::FdFlag::FD_CLOEXEC),
            )
            .unwrap();
            let new_stream = Arc::new(Mutex::new(stream));
            srvc.notifications = Some(new_stream.clone());
            new_stream
        }
    };

    let child_stdout = if let Some(fd) = &srvc.stdout_dup {
        fd.1
    }else{
        let (r,w) = nix::unistd::pipe().unwrap();
        srvc.stdout_dup = Some((r,w));
        w
    };
    let child_stderr = if let Some(fd) = &srvc.stderr_dup {
        fd.1
    }else{
        let (r,w) = nix::unistd::pipe().unwrap();
        srvc.stderr_dup = Some((r,w));
        w
    };

    // make sure we have the lock that the child will need
    let sockets_lock = sockets.lock().unwrap();
    let stream_locked = &*stream.lock().unwrap();
    match nix::unistd::fork() {
        Ok(nix::unistd::ForkResult::Parent { child, .. }) => {
            std::mem::drop(sockets_lock);
            after_fork_parent(
                srvc,
                name,
                child,
                std::path::Path::new(notify_socket_env_var.to_str().unwrap()),
                stream_locked,
            );
        }
        Ok(nix::unistd::ForkResult::Child) => {
            after_fork_child(
                srvc,
                &name,
                &*sockets_lock,
                notify_socket_env_var.to_str().unwrap(),
                child_stdout,
                child_stderr,
            );
        }
        Err(e) => error!("Fork for service: {} failed with: {}", name, e),
    }
}

pub fn start_service(
    srvc: &mut Service,
    name: String,
    sockets: ArcMutServiceTable,
    notification_socket_path: std::path::PathBuf,
) {
    srvc.status = ServiceStatus::Starting;
    start_service_with_filedescriptors(srvc, name, sockets, notification_socket_path);
    srvc.runtime_info.up_since = Some(std::time::Instant::now());
}
