use log::error;
use log::info;
use log::trace;
use log::warn;

use crate::runtime_info::*;
use crate::units::*;

fn get_next_service_to_shutdown(unit_table: &UnitTable) -> Option<UnitId> {
    for (_, unit) in unit_table.iter() {
        let status = &unit.common.status;
        {
            let status_locked = status.read().unwrap();
            if !(*status_locked).is_started() {
                continue;
            }
        }

        let kill_before = unit
            .common
            .dependencies
            .before
            .iter()
            .cloned()
            .filter(|next_id| {
                let unit = unit_table.get(next_id).unwrap();
                let status = &unit.common.status;
                let status_locked = status.read().unwrap();
                status_locked.is_started()
            })
            .collect::<Vec<_>>();
        if kill_before.is_empty() {
            trace!("Chose unit: {}", unit.id.name);
            return Some(unit.id.clone());
        } else {
            trace!(
                "Dont kill service {} yet. These Units depend on it: {:?}",
                unit.id.name,
                kill_before
            );
        }
    }
    None
}

fn shutdown_unit(shutdown_id: &UnitId, run_info: &RuntimeInfo) {
    let unit = run_info.unit_table.get(shutdown_id).unwrap();
    {
        trace!("Set unit status: {}", unit.id.name);
        let mut status_locked = unit.common.status.write().unwrap();
        *status_locked = UnitStatus::Stopping;
    }
    match &unit.specific {
        Specific::Service(specific) => {
            let mut_state = &mut *specific.state.write().unwrap();
            let kill_res =
                mut_state
                    .srvc
                    .kill(&specific.conf, unit.id.clone(), &unit.id.name, run_info);
            match kill_res {
                Ok(()) => {
                    trace!("Killed service unit: {}", unit.id.name);
                }
                Err(e) => error!("{}", e),
            }
            if let Some(datagram) = &mut_state.srvc.notifications {
                match datagram.shutdown(std::net::Shutdown::Both) {
                    Ok(()) => {
                        trace!(
                            "Closed notification socket for service unit: {}",
                            unit.id.name
                        );
                    }
                    Err(e) => error!(
                        "Error closing notification socket for service unit {}: {}",
                        unit.id.name, e
                    ),
                }
            }
            mut_state.srvc.notifications = None;

            if let Some(note_sock_path) = &mut_state.srvc.notifications_path {
                if note_sock_path.exists() {
                    match std::fs::remove_file(note_sock_path) {
                        Ok(()) => {
                            trace!(
                                "Removed notification socket for service unit: {}",
                                unit.id.name
                            );
                        }
                        Err(e) => error!(
                            "Error removing notification socket for service unit {}: {}",
                            unit.id.name, e
                        ),
                    }
                }
            }
        }
        Specific::Socket(specific) => {
            let mut_state = &mut *specific.state.write().unwrap();
            trace!("Close socket unit: {}", unit.id.name);
            match mut_state.sock.close_all(
                &specific.conf,
                unit.id.name.clone(),
                &mut *run_info.fd_store.write().unwrap(),
            ) {
                Err(e) => error!("Error while closing sockets: {}", e),
                Ok(()) => {}
            }
            trace!("Closed socket unit: {}", unit.id.name);
        }
        Specific::Target(_) => {
            // Nothing to do
        }
    }
    {
        trace!("Set unit status: {}", unit.id.name);
        let mut status_locked = unit.common.status.write().unwrap();
        *status_locked = UnitStatus::Stopped(StatusStopped::StoppedFinal, vec![]);
    }
}

static SHUTTING_DOWN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
// TODO maybe this should be available everywhere for situations where normally a panic would occur?
pub fn shutdown_sequence(run_info: ArcMutRuntimeInfo) {
    if SHUTTING_DOWN
        .compare_exchange(
            false,
            true,
            std::sync::atomic::Ordering::SeqCst,
            std::sync::atomic::Ordering::SeqCst,
        )
        .is_err()
    {
        // is already shutting down. Exit the process.
        warn!("Got a second termination signal. Exiting potentially dirty");
        std::process::exit(0);
    }

    std::thread::spawn(move || {
        trace!("Shutting down");
        let run_info_lock = match run_info.read() {
            Ok(r) => r,
            Err(e) => e.into_inner(),
        };
        let run_info_locked = &*run_info_lock;

        trace!("Kill all units");
        loop {
            let id = {
                if let Some(id) = get_next_service_to_shutdown(&run_info_locked.unit_table) {
                    id
                } else {
                    break;
                }
            };
            shutdown_unit(&id, run_info_locked);
        }
        trace!("Killed all units");

        let control_socket = run_info_locked
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

        #[cfg(feature = "cgroups")]
        {
            let _ = crate::platform::cgroups::move_out_of_own_cgroup(&std::path::PathBuf::from(
                "/sys/fs/cgroup/unified",
            ))
            .map_err(|e| error!("Error while cleaning up cgroups: {}", e));
        }

        info!("Shutdown finished");
        std::process::exit(0);
    });
}
