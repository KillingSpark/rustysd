use super::fork_child;
use super::fork_parent;
use super::pre_fork;
use crate::fd_store::FDStore;
use crate::services::Service;

fn start_service_with_filedescriptors(
    srvc: &mut Service,
    name: String,
    fd_store: &FDStore,
    notification_socket_path: std::path::PathBuf,
) -> Result<(), String> {
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
        return Err(format!(
            "The service {} specified an executable that does not exist: {:?}",
            name, &cmd
        ));
    }
    if !cmd.is_file() {
        error!(
            "The service {} specified an executable that is not a file: {:?}",
            name, &cmd
        );
        return Err(format!(
            "The service {} specified an executable that is not a file: {:?}",
            name, &cmd
        ));
    }

    // 1. fork
    // 2. in fork use dup2 to map all relevant file desrciptors to 3..x
    // 3. in fork mark all other file descriptors with FD_CLOEXEC
    // 4. set relevant env varibales $LISTEN_FDS $LISTEN_PID
    // 4. execve the cmd with the args

    let prefork_res = pre_fork::pre_fork(srvc, &name, &notification_socket_path)?;

    // make sure we have the lock that the child will need
    let stream_locked = &*prefork_res.notification_socket.lock().unwrap();
    match nix::unistd::fork() {
        Ok(nix::unistd::ForkResult::Parent { child, .. }) => {
            fork_parent::after_fork_parent(
                srvc,
                name,
                child,
                std::path::Path::new(prefork_res.notify_socket_env_var.to_str().unwrap()),
                stream_locked,
            );
        }
        Ok(nix::unistd::ForkResult::Child) => {
            fork_child::after_fork_child(
                srvc,
                &name,
                fd_store,
                prefork_res.notify_socket_env_var.to_str().unwrap(),
                prefork_res.stdout,
                prefork_res.stderr,
            );
        }
        Err(e) => error!("Fork for service: {} failed with: {}", name, e),
    }
    Ok(())
}

pub fn start_service(
    srvc: &mut Service,
    name: String,
    fd_store: &FDStore,
    notification_socket_path: std::path::PathBuf,
) -> Result<(), String> {
    if let Some(conf) = &srvc.service_config {
        if conf.accept {
            warn!("Inetd style accepting is not supported");
            Err("Inetd style accepting is not supported".into())
        } else {
            if srvc.pid.is_some() {
                return Err(format!(
                    "Service {} has already a pid {:?}",
                    name,
                    srvc.pid.unwrap()
                ));
            }
            if srvc.process_group.is_some() {
                return Err(format!(
                    "Service {} has already a pid {:?}",
                    name,
                    srvc.process_group.unwrap()
                ));
            }
            start_service_with_filedescriptors(srvc, name, fd_store, notification_socket_path)?;
            srvc.runtime_info.up_since = Some(std::time::Instant::now());
            Ok(())
        }
    } else {
        unreachable!();
    }
}
