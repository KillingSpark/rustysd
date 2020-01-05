use super::fork_child;
use crate::fd_store::FDStore;
use crate::services::Service;

fn start_service_with_filedescriptors(
    srvc: &mut Service,
    name: &str,
    fd_store: &FDStore,
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

    // make sure we have the lock that the child will need
    match nix::unistd::fork() {
        Ok(nix::unistd::ForkResult::Parent { child, .. }) => {
            srvc.pid = Some(child);
            srvc.process_group = Some(nix::unistd::Pid::from_raw(-child.as_raw()));
        }
        Ok(nix::unistd::ForkResult::Child) => {
            let notifications_path = {
                if let Some(p) = &srvc.notifications_path {
                    p.to_str().unwrap().to_owned()
                } else {
                    unreachable!();
                }
            };
            let stdout = {
                if let Some(rwpair) = &srvc.stdout_dup {
                    rwpair.1
                } else {
                    unreachable!();
                }
            };
            let stderr = {
                if let Some(rwpair) = &srvc.stderr_dup {
                    rwpair.1
                } else {
                    unreachable!();
                }
            };
            fork_child::after_fork_child(srvc, &name, fd_store, &notifications_path, stdout, stderr);
        }
        Err(e) => error!("Fork for service: {} failed with: {}", name, e),
    }
    Ok(())
}

pub fn start_service(
    srvc: &mut Service,
    name: &str,
    fd_store: &FDStore,
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
            start_service_with_filedescriptors(srvc, name, fd_store)?;
            srvc.runtime_info.up_since = Some(std::time::Instant::now());
            Ok(())
        }
    } else {
        unreachable!();
    }
}
