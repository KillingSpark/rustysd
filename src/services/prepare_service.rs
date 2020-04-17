use super::StdIo;
use crate::services::Service;
use crate::units::ServiceConfig;
use crate::units::StdIoOption;
use std::os::unix::io::AsRawFd;
use std::os::unix::net::UnixDatagram;

fn open_stdio(setting: &Option<StdIoOption>) -> Result<StdIo, String> {
    match setting {
        Some(StdIoOption::File(path)) => {
            let file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .read(true)
                .open(path)
                .map_err(|e| format!("Error opening file: {:?}: {}", path, e))?;
            Ok(StdIo::File(file))
        }
        Some(StdIoOption::AppendFile(path)) => {
            let file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .read(true)
                .open(path)
                .map_err(|e| format!("Error opening file: {:?}: {}", path, e))?;
            Ok(StdIo::File(file))
        }
        None => {
            let (r, w) = nix::unistd::pipe().unwrap();
            Ok(super::StdIo::Piped(r, w))
        }
    }
}

pub fn prepare_service(
    srvc: &mut Service,
    conf: &ServiceConfig,
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

    if srvc.stdout.is_none() {
        srvc.stdout = Some(open_stdio(&conf.exec_config.stdout_path)?);
    }
    if srvc.stderr.is_none() {
        srvc.stderr = Some(open_stdio(&conf.exec_config.stderr_path)?);
    }

    srvc.notifications_path = Some(notify_socket_env_var);

    Ok(())
}
