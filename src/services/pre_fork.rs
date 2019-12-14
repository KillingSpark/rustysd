use crate::services::Service;
use std::os::unix::io::AsRawFd;
use std::os::unix::io::RawFd;
use std::os::unix::net::UnixDatagram;
use std::sync::{Arc, Mutex};

pub struct Preforkresult {
    pub notification_socket: Arc<Mutex<UnixDatagram>>,
    pub notify_socket_env_var: std::path::PathBuf,
    pub stdout: RawFd,
    pub stderr: RawFd,
}

pub fn pre_fork(
    srvc: &mut Service,
    name: &String,
    notification_socket_path: &std::path::PathBuf,
) -> Preforkresult {
    // setup socket for notifications from the service
    if !notification_socket_path.exists() {
        std::fs::create_dir_all(notification_socket_path).unwrap();
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

    Preforkresult{
        notification_socket: stream,
        notify_socket_env_var,
        stdout: child_stdout,
        stderr: child_stderr,
    }
}
