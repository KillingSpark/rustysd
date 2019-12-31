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
    'outer: loop {
        // Pick up new signals
        for signal in signals.forever() {
            match signal as libc::c_int {
                signal_hook::SIGCHLD => {
                    std::iter::from_fn(get_next_exited_child)
                        .take_while(Result::is_ok)
                        .for_each(|val| {
                            let note_sock_path = notification_socket_path.clone();
                            let eventfds_clone = eventfds.clone();
                            let run_info_clone = run_info.clone();
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
                }
                signal_hook::SIGTERM | signal_hook::SIGINT | signal_hook::SIGQUIT => {
                    trace!("Shutting down");
                    trace!("Get unit lock");
                    let unit_table_locked = run_info.unit_table.write().unwrap();
                    trace!("Kill all services");
                    for (id, unit) in unit_table_locked.iter() {
                        if id.0 != UnitIdKind::Service {
                            continue;
                        }
                        trace!("Lock to kill service unit: {}", id);
                        let unit_locked = &mut *unit.lock().unwrap();
                        match &mut unit_locked.specialized {
                            UnitSpecialized::Service(srvc) => {
                                {
                                    trace!("Get status lock");
                                    let status_table_locked =
                                        run_info.status_table.write().unwrap();
                                    trace!("Set service status: {}", unit_locked.conf.name());
                                    let status = status_table_locked.get(&unit_locked.id).unwrap();
                                    let mut status_locked = status.lock().unwrap();
                                    *status_locked = UnitStatus::Stopping;
                                }
                                {
                                    trace!("Get pid lock");
                                    let pid_table_locked = &mut *run_info.pid_table.lock().unwrap();
                                    trace!("Kill service unit: {}", unit_locked.conf.name());
                                    srvc.kill(
                                        unit_locked.id,
                                        &unit_locked.conf.name(),
                                        pid_table_locked,
                                    );
                                    trace!("Killed service unit: {}", unit_locked.conf.name());
                                }
                                {
                                    trace!("Get status lock");
                                    let status_table_locked =
                                        run_info.status_table.write().unwrap();
                                    trace!("Set service status: {}", unit_locked.conf.name());
                                    let status = status_table_locked.get(&unit_locked.id).unwrap();
                                    let mut status_locked = status.lock().unwrap();
                                    *status_locked = UnitStatus::Stopping;
                                }
                            }
                            UnitSpecialized::Socket(_) => {
                                // closed below
                            }
                            UnitSpecialized::Target => {
                                // Nothing to do
                            }
                        }
                    }
                    trace!("Killed all services");
                    for (id, unit) in unit_table_locked.iter() {
                        if id.0 != UnitIdKind::Socket {
                            continue;
                        }

                        trace!("Lock to close socket unit: {}", id);
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
                                trace!("Closed socket unit: {}", unit_locked.conf.name());
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

#[derive(Debug)]
pub enum ChildTermination {
    Signal(nix::sys::signal::Signal),
    Exit(i32),
}
type ChildIterElem = Result<(nix::unistd::Pid, ChildTermination), nix::Error>;

fn get_next_exited_child() -> Option<ChildIterElem> {
    let wait_any_pid = nix::unistd::Pid::from_raw(-1);
    match nix::sys::wait::waitpid(wait_any_pid, Some(nix::sys::wait::WaitPidFlag::WNOHANG)) {
        Ok(exit_status) => match exit_status {
            nix::sys::wait::WaitStatus::Exited(pid, code) => {
                Some(Ok((pid, ChildTermination::Exit(code))))
            }
            nix::sys::wait::WaitStatus::Signaled(pid, signal, _dumped_core) => {
                // signals get handed to the parent if the child got killed by it but didnt handle the
                // signal itself
                // we dont care if the service dumped it's core
                Some(Ok((pid, ChildTermination::Signal(signal))))
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
