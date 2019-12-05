use crate::services;
use crate::units::*;
use signal_hook::iterator::Signals;

pub fn handle_signals(
    service_table: ArcMutServiceTable,
    socket_table: ArcMutSocketTable,
    pid_table: ArcMutPidTable,
    notification_socket_path: std::path::PathBuf,
) {
    let signals =
        Signals::new(&[signal_hook::SIGCHLD]).expect("Couldnt setup listening to the signals");

    loop {
        // Pick up new signals
        for signal in signals.forever() {
            match signal as libc::c_int {
                signal_hook::SIGCHLD => {
                    std::iter::from_fn(get_next_exited_child)
                        .take_while(Result::is_ok)
                        .for_each(|val| match val {
                            Ok((pid, code)) => services::service_exit_handler(
                                pid,
                                code,
                                service_table.clone(),
                                pid_table.clone(),
                                socket_table.clone(),
                                notification_socket_path.clone(),
                            ),
                            Err(e) => {
                                error!("{}", e);
                            }
                        });
                }

                _ => unreachable!(),
            }
        }
    }
}

fn get_next_exited_child() -> Option<Result<(nix::unistd::Pid, i32), nix::Error>> {
    let wait_any_pid = nix::unistd::Pid::from_raw(-1);
    match nix::sys::wait::waitpid(wait_any_pid, Some(nix::sys::wait::WaitPidFlag::WNOHANG)) {
        Ok(exit_status) => match exit_status {
            nix::sys::wait::WaitStatus::Exited(pid, code) => Some(Ok((pid, code))),
            nix::sys::wait::WaitStatus::Signaled(pid, signal, _dumped_core) => {
                // signals get handed to the parent if the child got killed by it but didnt handle the
                // signal itself
                if signal == nix::sys::signal::SIGTERM {
                    // we dont care if the service dumped it's core
                    Some(Ok((pid, 0)))
                } else {
                    None
                }
            }
            nix::sys::wait::WaitStatus::StillAlive => {
                trace!("No more state changes to poll");
                None
            }
            _ => {
                trace!("Child signaled with code: {:?}", exit_status);
                None
            }
        },
        Err(e) => {
            if let nix::Error::Sys(nix::errno::Errno::ECHILD) = e {
            } else {
                trace!("Error while waiting: {}", e);
            }
            Some(Err(e))
        }
    }
}
