use log::error;
use log::trace;
use which::which;

use super::fork_child;
use crate::fd_store::FDStore;
use crate::services::RunCmdError;
use crate::services::Service;
use crate::units::ServiceConfig;

use std::path::Path;

fn start_service_with_filedescriptors(
    self_path: &Path,
    srvc: &mut Service,
    conf: &ServiceConfig,
    name: &str,
    fd_store: &FDStore,
) -> Result<(), RunCmdError> {
    // check if executable even exists
    let cmd = which(&conf.exec.cmd).map_err(|err| {
        RunCmdError::SpawnError(
            name.to_owned(),
            format!("Could not resolve command to an exectuable file: {err:?}"),
        )
    })?;
    if !cmd.exists() {
        error!(
            "The service {} specified an executable that does not exist: {:?}",
            name, &conf.exec.cmd
        );
        return Err(RunCmdError::SpawnError(
            conf.exec.cmd.clone(),
            format!("Executable does not exist"),
        ));
    }
    if !cmd.is_file() {
        error!(
            "The service {} specified an executable that is not a file: {:?}",
            name, &cmd
        );
        return Err(RunCmdError::SpawnError(
            conf.exec.cmd.clone(),
            format!("Executable does not exist (is a directory)"),
        ));
    }

    // 1. fork
    // 1. in fork use dup2 to map all relevant file desrciptors to 3..x
    // 1. in fork mark all other file descriptors with FD_CLOEXEC
    // 1. in fork set relevant env varibales $LISTEN_FDS $LISTEN_PID
    // 1. in fork execve the cmd with the args
    // 1. in parent set pid and return. Waiting will be done afterwards if necessary

    let notifications_path = {
        if let Some(p) = &srvc.notifications_path {
            p.to_str().unwrap().to_owned()
        } else {
            return Err(RunCmdError::Generic(format!(
                "Tried to start service: {} without a notifications path",
                name,
            )));
        }
    };

    super::fork_os_specific::pre_fork_os_specific(conf).map_err(|e| RunCmdError::Generic(e))?;

    let mut fds = Vec::new();
    let mut names = Vec::new();

    for socket in &conf.sockets {
        let sock_fds = fd_store
            .get_global(&socket.name)
            .unwrap()
            .iter()
            .map(|(_, _, fd)| fd.as_raw_fd())
            .collect::<Vec<_>>();

        let sock_names = fd_store
            .get_global(&socket.name)
            .unwrap()
            .iter()
            .map(|(_, name, _)| name.clone())
            .collect::<Vec<_>>();

        fds.extend(sock_fds);
        names.extend(sock_names);
    }

    // We first exec into our own executable again and apply this config
    // We transfer the config via a anonymous shared memory file
    let exec_helper_conf = crate::entrypoints::ExecHelperConfig {
        name: name.to_owned(),
        cmd: cmd,
        args: conf.exec.args.clone(),
        env: vec![
            ("LISTEN_FDS".to_owned(), format!("{}", names.len())),
            ("LISTEN_FDNAMES".to_owned(), names.join(":")),
            ("NOTIFY_SOCKET".to_owned(), notifications_path.clone()),
        ],
        group: conf.exec_config.group.as_raw(),
        supplementary_groups: conf
            .exec_config
            .supplementary_groups
            .iter()
            .map(|gid| gid.as_raw())
            .collect(),
        user: conf.exec_config.user.as_raw(),

        platform_specific: conf.platform_specific.clone(),
    };

    let marshalled_config = serde_json::to_string(&exec_helper_conf).unwrap();

    // crate the shared memory file
    let exec_helper_conf_fd = shmemfdrs::create_shmem(
        &std::ffi::CString::new(name).unwrap(),
        marshalled_config.as_bytes().len() + 1,
    );
    if exec_helper_conf_fd < 0 {
        return Err(RunCmdError::CreatingShmemFailed(
            name.to_owned(),
            std::io::Error::from_raw_os_error(exec_helper_conf_fd).kind(),
        ));
    }
    use std::os::unix::io::FromRawFd;
    let mut exec_helper_conf_file = unsafe { std::fs::File::from_raw_fd(exec_helper_conf_fd) };

    // write the config to the file
    use std::io::Write;
    exec_helper_conf_file
        .write_all(marshalled_config.as_bytes())
        .unwrap();
    exec_helper_conf_file.write(&[b'\n']).unwrap();
    use std::io::Seek;
    exec_helper_conf_file
        .seek(std::io::SeekFrom::Start(0))
        .unwrap();

    // need to allocate this before forking. Currently this is just static info, we could only do this once...
    let self_path_cstr = std::ffi::CString::new(self_path.to_str().unwrap()).unwrap();
    let name_arg = std::ffi::CString::new("exec_helper").unwrap();
    let self_args = [name_arg.as_ptr(), std::ptr::null()];

    trace!("Start main executable for service: {name}: {:?} {:?}", exec_helper_conf.cmd, exec_helper_conf.args);
    match unsafe { nix::unistd::fork() } {
        Ok(nix::unistd::ForkResult::Parent { child, .. }) => {
            // make sure the file exists until after we fork before closing it
            drop(exec_helper_conf_file);
            srvc.pid = Some(child);
            srvc.process_group = Some(nix::unistd::Pid::from_raw(-child.as_raw()));
        }
        Ok(nix::unistd::ForkResult::Child) => {
            let stdout = {
                if let Some(stdio) = &srvc.stdout {
                    stdio.write_fd()
                } else {
                    unreachable!();
                }
            };
            let stderr = {
                if let Some(stdio) = &srvc.stderr {
                    stdio.write_fd()
                } else {
                    unreachable!();
                }
            };

            fork_child::after_fork_child(
                &self_path_cstr,
                self_args.as_slice(),
                &mut fds,
                stdout,
                stderr,
                exec_helper_conf_fd,
            );
        }
        Err(e) => error!("Fork for service: {} failed with: {}", name, e),
    }
    Ok(())
}

pub fn start_service(
    self_path: &Path,
    srvc: &mut Service,
    conf: &ServiceConfig,
    name: &str,
    fd_store: &FDStore,
) -> Result<(), super::RunCmdError> {
    start_service_with_filedescriptors(self_path, srvc, conf, name, fd_store)?;
    Ok(())
}
