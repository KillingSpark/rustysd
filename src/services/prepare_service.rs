use crate::services::Service;
use std::os::unix::io::AsRawFd;
use std::os::unix::net::UnixDatagram;

pub fn prepare_service(
    srvc: &mut Service,
    name: &str,
    notification_socket_path: &std::path::PathBuf,
) -> Result<(), String> {
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

    if srvc.notifications.is_none() {
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

        srvc.notifications = Some(stream);
    }

    if srvc.stdout_dup.is_none() {
        let (r, w) = nix::unistd::pipe().unwrap();
        srvc.stdout_dup = Some((r, w));
    }
    if srvc.stderr_dup.is_none() {
        let (r, w) = nix::unistd::pipe().unwrap();
        srvc.stderr_dup = Some((r, w));
    }

    srvc.notifications_path = Some(notify_socket_env_var);

    Ok(())
}
