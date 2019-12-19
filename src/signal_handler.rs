//! Handle signals send to this process from either the outside or the child processes

use crate::services;
use crate::units::*;
use signal_hook::iterator::Signals;

pub fn handle_signals(
    signals: Signals,
    unit_table: ArcMutUnitTable,
    pid_table: ArcMutPidTable,
    notification_socket_path: std::path::PathBuf,
) {
    'outer: loop {
        // Pick up new signals
        for signal in signals.forever() {
            match signal as libc::c_int {
                signal_hook::SIGCHLD => {
                    std::iter::from_fn(get_next_exited_child)
                        .take_while(Result::is_ok)
                        .for_each(|val| match val {
                            Ok((pid, code)) => match services::service_exit_handler(
                                pid,
                                code,
                                unit_table.clone(),
                                pid_table.clone(),
                                notification_socket_path.clone(),
                            ) {
                                Ok(()) => { /* Happy */ }
                                Err(e) => {
                                    error!("{}", e);
                                }
                            },
                            Err(e) => {
                                error!("{}", e);
                            }
                        });
                }
                signal_hook::SIGTERM | signal_hook::SIGINT | signal_hook::SIGQUIT => {
                    // TODO kill all services
                    // TODO close all notification sockets
                    // TODO close all other sockets
                    println!("Received termination signal. Rustysd checking out.");
                    break 'outer;
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
