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
    loop {
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
                                Ok((pid, code)) => services::service_exit_handler_new_thread(
                                    pid,
                                    code,
                                    run_info_clone,
                                    note_sock_path,
                                    eventfds_clone,
                                ),
                                Err(e) => {
                                    error!("{}", e);
                                }
                            }
                        });
                }
                signal_hook::SIGTERM | signal_hook::SIGINT | signal_hook::SIGQUIT => {
                    println!("Received termination signal. Rustysd checking out");
                    shutdown_sequence(run_info.clone());
                }

                _ => unreachable!(),
            }
        }
    }
}

// TODO maybe this should be available everywhere for situations where normally a panic would occur?
pub fn shutdown_sequence(run_info: ArcRuntimeInfo) {
    std::thread::spawn(move || {
        trace!("Shutting down");
        trace!("Get unit lock");

        // Here we need to get the locks regardless of posions.
        // At least try to shutdown as much as possible as cleanly as possible
        let unit_table_locked = match run_info.unit_table.write() {
            Ok(lock) => lock,
            Err(err) => err.into_inner(),
        };

        trace!("Kill all services");
        for (id, unit) in unit_table_locked.iter() {
            if id.0 != UnitIdKind::Service {
                continue;
            }
            trace!("Lock to kill service unit: {}", id);
            let unit_locked = &mut *match unit.lock() {
                Ok(lock) => lock,
                Err(err) => err.into_inner(),
            };
            match &mut unit_locked.specialized {
                UnitSpecialized::Service(srvc) => {
                    {
                        trace!("Get status lock");
                        let status_table_locked = match run_info.status_table.write() {
                            Ok(lock) => lock,
                            Err(err) => err.into_inner(),
                        };
                        trace!("Set service status: {}", unit_locked.conf.name());
                        let status = status_table_locked.get(&unit_locked.id).unwrap();
                        let mut status_locked = status.lock().unwrap();
                        *status_locked = UnitStatus::Stopping;
                    }
                    {
                        let kill_res =
                            srvc.kill(unit_locked.id, &unit_locked.conf.name(), run_info.clone());
                        match kill_res {
                            Ok(()) => {
                                trace!("Killed service unit: {}", unit_locked.conf.name());
                            }
                            Err(e) => error!("{}", e),
                        }
                        if let Some(datagram) = &srvc.notifications {
                            match datagram.shutdown(std::net::Shutdown::Both) {
                                Ok(()) => {
                                    trace!(
                                        "Closed notification socket for service unit: {}",
                                        unit_locked.conf.name()
                                    );
                                }
                                Err(e) => error!(
                                    "Error closing notification socket for service unit {}: {}",
                                    unit_locked.conf.name(),
                                    e
                                ),
                            }
                        }
                        srvc.notifications = None;
                        if let Some(note_sock_path) = &srvc.notifications_path {
                            if note_sock_path.exists() {
                                match std::fs::remove_file(note_sock_path) {
                                    Ok(()) => {
                                        trace!(
                                            "Removed notification socket for service unit: {}",
                                            unit_locked.conf.name()
                                        );
                                    }
                                    Err(e) => error!(
                                        "Error removing notification socket for service unit {}: {}",
                                        unit_locked.conf.name(),
                                        e
                                    ),
                                }
                            }
                        }
                    }
                    {
                        trace!("Get status lock");
                        let status_table_locked = match run_info.status_table.write() {
                            Ok(lock) => lock,
                            Err(err) => err.into_inner(),
                        };
                        trace!("Set service status: {}", unit_locked.conf.name());
                        let status = status_table_locked.get(&unit_locked.id).unwrap();
                        let mut status_locked = status.lock().unwrap();
                        *status_locked = UnitStatus::StoppedFinal("Rustysd shutdown".into());
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
            let unit_locked = &mut *match unit.lock() {
                Ok(lock) => lock,
                Err(err) => err.into_inner(),
            };
            match &mut unit_locked.specialized {
                UnitSpecialized::Service(_) => {
                    // killed above
                }
                UnitSpecialized::Socket(sock) => {
                    {
                        trace!("Get status lock");
                        let status_table_locked = match run_info.status_table.write() {
                            Ok(lock) => lock,
                            Err(err) => err.into_inner(),
                        };
                        trace!("Set service status: {}", unit_locked.conf.name());
                        let status = status_table_locked.get(&unit_locked.id).unwrap();
                        let mut status_locked = status.lock().unwrap();
                        *status_locked = UnitStatus::Stopping;
                    }
                    {
                        trace!("Close socket unit: {}", unit_locked.conf.name());
                        match sock.close_all(
                            unit_locked.conf.name(),
                            &mut *run_info.fd_store.write().unwrap(),
                        ) {
                            Err(e) => error!("Error while closing sockets: {}", e),
                            Ok(()) => {}
                        }
                        trace!("Closed socket unit: {}", unit_locked.conf.name());
                    }
                    {
                        trace!("Get status lock");
                        let status_table_locked = match run_info.status_table.write() {
                            Ok(lock) => lock,
                            Err(err) => err.into_inner(),
                        };
                        trace!("Set service status: {}", unit_locked.conf.name());
                        let status = status_table_locked.get(&unit_locked.id).unwrap();
                        let mut status_locked = status.lock().unwrap();
                        *status_locked = UnitStatus::StoppedFinal("Rustysd shutdown".into());
                    }
                }
                UnitSpecialized::Target => {
                    // Nothing to do
                }
            }
        }
        let control_socket = run_info
            .config
            .notification_sockets_dir
            .join("control.socket");
        if control_socket.exists() {
            match std::fs::remove_file(control_socket) {
                Ok(()) => {
                    trace!("Removed control socket");
                }
                Err(e) => error!("Error removing control socket: {}", e),
            }
        }
        println!("Shutdown finished");
        std::process::exit(0);
    });
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum ChildTermination {
    Signal(nix::sys::signal::Signal),
    Exit(i32),
}

impl ChildTermination {
    pub fn success(&self) -> bool {
        match self {
            ChildTermination::Signal(_) => false,
            ChildTermination::Exit(code) => *code == 0,
        }
    }
}

type ChildIterElem = Result<(nix::unistd::Pid, ChildTermination), nix::Error>;

fn get_next_exited_child() -> Option<ChildIterElem> {
    let wait_any_pid = nix::unistd::Pid::from_raw(-1);
    let wait_flags = nix::sys::wait::WaitPidFlag::WNOHANG;
    match nix::sys::wait::waitpid(wait_any_pid, Some(wait_flags)) {
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
                trace!("Ignored child signal received with code: {:?}", exit_status);
                // return next child, we dont care about other events like stop/continue of children
                get_next_exited_child()
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
