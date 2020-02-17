use super::fork_child;
use crate::fd_store::FDStore;
use crate::services::RunCmdError;
use crate::services::Service;

fn start_service_with_filedescriptors(
    srvc: &mut Service,
    name: &str,
    fd_store: &FDStore,
) -> Result<(), RunCmdError> {
    // check if executable even exists
    let cmd = std::path::PathBuf::from(&srvc.service_config.exec.cmd);
    if !cmd.exists() {
        error!(
            "The service {} specified an executable that does not exist: {:?}",
            name, &srvc.service_config.exec.cmd
        );
        return Err(RunCmdError::SpawnError(
            srvc.service_config.exec.cmd.clone(),
            format!("Executable does not exist"),
        ));
    }
    if !cmd.is_file() {
        error!(
            "The service {} specified an executable that is not a file: {:?}",
            name, &cmd
        );
        return Err(RunCmdError::SpawnError(
            srvc.service_config.exec.cmd.clone(),
            format!("Executable does not exist (is a directory)"),
        ));
    }

    // 1. fork
    // 1. in fork use dup2 to map all relevant file desrciptors to 3..x
    // 1. in fork mark all other file descriptors with FD_CLOEXEC
    // 1. in fork set relevant env varibales $LISTEN_FDS $LISTEN_PID
    // 1. in fork execve the cmd with the args
    // 1. in parent set pid and return. Waiting will be done afterwards if necessary

    super::fork_os_specific::pre_fork_os_specific(srvc).map_err(|e| RunCmdError::Generic(e))?;

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
            fork_child::after_fork_child(
                srvc,
                &name,
                fd_store,
                &notifications_path,
                stdout,
                stderr,
            );
        }
        Err(e) => error!("Fork for service: {} failed with: {}", name, e),
    }
    Ok(())
}

pub fn start_service(
    srvc: &mut Service,
    name: &str,
    fd_store: &FDStore,
) -> Result<(), super::RunCmdError> {
    start_service_with_filedescriptors(srvc, name, fd_store)?;
    srvc.runtime_info.up_since = Some(std::time::Instant::now());
    Ok(())
}
