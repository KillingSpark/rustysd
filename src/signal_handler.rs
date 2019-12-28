//! Handle signals send to this process from either the outside or the child processes

use crate::platform::EventFd;
use crate::services;
use crate::units::*;
use signal_hook::iterator::Signals;

pub fn handle_signals(
    signals: Signals,
    run_info: ArcRuntimeInfo,
    notification_socket_path: std::path::PathBuf,
    eventfds: Vec<EventFd>,
) {
    let tpool = threadpool::ThreadPool::new(6);
    'outer: loop {
        // Pick up new signals
        for signal in signals.forever() {
            match signal as libc::c_int {
                signal_hook::SIGCHLD => {
                    std::iter::from_fn(get_next_exited_child)
                        .take_while(Result::is_ok)
                        .for_each(|val| {
                            let note_sock_path = notification_socket_path.clone();
                            let eventfds_clone =eventfds.clone();
                            let run_info_clone =run_info.clone();
                            tpool.execute(move || {
                                match val {
                                    Ok((pid, code)) => match services::service_exit_handler(
                                        pid,
                                        code,
                                        run_info_clone,
                                        note_sock_path,
                                        &eventfds_clone,
                                    ) {
                                        Ok(()) => { /* Happy */ }
                                        Err(e) => {
                                            error!("{}", e);
                                        }
                                    },
                                    Err(e) => {
                                        error!("{}", e);
                                    }
                                }
                            });
                        });
                }
                signal_hook::SIGTERM | signal_hook::SIGINT | signal_hook::SIGQUIT => {
                    for unit in run_info.unit_table.read().unwrap().values() {
                        let unit_locked = &mut *unit.lock().unwrap();
                        match &mut unit_locked.specialized {
                            UnitSpecialized::Service(srvc) => {
                                trace!("Kill service unit: {}", unit_locked.conf.name());
                                let pid_table_locked = &mut *run_info.pid_table.lock().unwrap();
                                let status_table_locked = &*run_info.status_table.read().unwrap();
                                srvc.kill_final(
                                    unit_locked.id,
                                    &unit_locked.conf.name(),
                                    pid_table_locked,
                                    status_table_locked,
                                );
                            }
                            UnitSpecialized::Socket(_) => {
                                // closed below
                            }
                            UnitSpecialized::Target => {
                                // Nothing to do
                            }
                        }
                    }
                    for unit in run_info.unit_table.read().unwrap().values() {
                        let unit_locked = &mut *unit.lock().unwrap();
                        match &mut unit_locked.specialized {
                            UnitSpecialized::Service(_) => {
                                // killed above
                            }
                            UnitSpecialized::Socket(sock) => {
                                trace!("Close socket unit: {}", unit_locked.conf.name());
                                match sock.close_all() {
                                    Err(e) => error!("Error while closing sockets: {}", e),
                                    Ok(()) => {}
                                }
                            }
                            UnitSpecialized::Target => {
                                // Nothing to do
                            }
                        }
                    }
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
