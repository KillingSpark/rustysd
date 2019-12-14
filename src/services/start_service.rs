use crate::services::{Service, ServiceStatus};
use crate::units::*;
use std::os::unix::io::AsRawFd;
use std::os::unix::net::UnixDatagram;

use super::fork_parent;
use super::fork_child;

use std::sync::{Arc, Mutex};


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
    } else {
        let (r, w) = nix::unistd::pipe().unwrap();
        srvc.stdout_dup = Some((r, w));
        w
    };
    let child_stderr = if let Some(fd) = &srvc.stderr_dup {
        fd.1
    } else {
        let (r, w) = nix::unistd::pipe().unwrap();
        srvc.stderr_dup = Some((r, w));
        w
    };

    // make sure we have the lock that the child will need
    let sockets_lock = sockets.lock().unwrap();
    let stream_locked = &*stream.lock().unwrap();
    match nix::unistd::fork() {
        Ok(nix::unistd::ForkResult::Parent { child, .. }) => {
            std::mem::drop(sockets_lock);
            fork_parent::after_fork_parent(
                srvc,
                name,
                child,
                std::path::Path::new(notify_socket_env_var.to_str().unwrap()),
                stream_locked,
            );
        }
        Ok(nix::unistd::ForkResult::Child) => {
            fork_child::after_fork_child(
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
    if let Some(conf) = &srvc.service_config {
        if conf.accept {
            warn!("Inetd style accepting is not supported");
            srvc.status = ServiceStatus::Stopped;
        } else {
            srvc.status = ServiceStatus::Starting;
            start_service_with_filedescriptors(srvc, name, sockets, notification_socket_path);
            srvc.runtime_info.up_since = Some(std::time::Instant::now());
        }
    } else {
        unreachable!();
    }
}
