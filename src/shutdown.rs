use crate::units::*;

fn get_next_service_to_shutdown(
    unit_table_locked: &UnitTable,
    status_table_locked: &StatusTable,
) -> Option<UnitId> {
    for (_, unit) in unit_table_locked.iter() {
        let unit_locked = &mut *match unit.lock() {
            Ok(lock) => lock,
            Err(err) => err.into_inner(),
        };
        let status = status_table_locked.get(&unit_locked.id).unwrap();
        {
            let status_locked = status.lock().unwrap();
            if !(*status_locked == UnitStatus::Started
                || *status_locked == UnitStatus::Starting
                || *status_locked == UnitStatus::StartedWaitingForSocket)
            {
                continue;
            }
        }

        let kill_before = unit_locked
            .install
            .before
            .iter()
            .copied()
            .filter(|next_id| {
                let status = status_table_locked.get(&next_id).unwrap();
                let status_locked = status.lock().unwrap();
                match *status_locked {
                    UnitStatus::Stopped | UnitStatus::StoppedFinal(_) => false,
                    _ => true,
                }
            })
            .collect::<Vec<_>>();
        if kill_before.is_empty() {
            trace!("Chose unit: {}", unit_locked.id.name);
            return Some(unit_locked.id);
        } else {
            trace!(
                "Dont kill service {} yet. These IDs depend on it: {:?}",
                unit_locked.id.name,
                kill_before
            );
        }
    }
    None
}

fn shutdown_unit(unit_locked: &mut Unit, run_info: ArcRuntimeInfo) {
    {
        trace!("Get status lock");
        let status_table_locked = match run_info.status_table.write() {
            Ok(lock) => lock,
            Err(err) => err.into_inner(),
        };
        trace!("Set unit status: {}", unit_locked.id.name);
        let status = status_table_locked.get(&unit_locked.id).unwrap();
        let mut status_locked = status.lock().unwrap();
        *status_locked = UnitStatus::Stopping;
    }
    match &mut unit_locked.specialized {
        UnitSpecialized::Service(srvc) => {
            let kill_res = srvc.kill(unit_locked.id, &unit_locked.id.name, run_info.clone());
            match kill_res {
                Ok(()) => {
                    trace!("Killed service unit: {}", unit_locked.id.name);
                }
                Err(e) => error!("{}", e),
            }
            if let Some(datagram) = &srvc.notifications {
                match datagram.shutdown(std::net::Shutdown::Both) {
                    Ok(()) => {
                        trace!(
                            "Closed notification socket for service unit: {}",
                            unit_locked.id.name
                        );
                    }
                    Err(e) => error!(
                        "Error closing notification socket for service unit {}: {}",
                        unit_locked.id.name,
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
                                unit_locked.id.name
                            );
                        }
                        Err(e) => error!(
                            "Error removing notification socket for service unit {}: {}",
                            unit_locked.id.name,
                            e
                        ),
                    }
                }
            }
        }
        UnitSpecialized::Socket(sock) => {
            trace!("Close socket unit: {}", unit_locked.id.name);
            match sock.close_all(
                unit_locked.id.name,
                &mut *run_info.fd_store.write().unwrap(),
            ) {
                Err(e) => error!("Error while closing sockets: {}", e),
                Ok(()) => {}
            }
            trace!("Closed socket unit: {}", unit_locked.id.name);
        }
        UnitSpecialized::Target => {
            // Nothing to do
        }
    }
    {
        trace!("Get status lock");
        let status_table_locked = match run_info.status_table.write() {
            Ok(lock) => lock,
            Err(err) => err.into_inner(),
        };
        trace!("Set unit status: {}", unit_locked.id.name);
        let status = status_table_locked.get(&unit_locked.id).unwrap();
        let mut status_locked = status.lock().unwrap();
        *status_locked = UnitStatus::StoppedFinal("Rustysd shutdown".into());
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

        trace!("Kill all units");
        loop {
            let id = {
                let status_table_locked = match run_info.status_table.write() {
                    Ok(lock) => lock,
                    Err(err) => err.into_inner(),
                };
                if let Some(id) =
                    get_next_service_to_shutdown(&*unit_table_locked, &*status_table_locked)
                {
                    id
                } else {
                    break;
                }
            };
            let unit = unit_table_locked.get(&id).unwrap();
            trace!("Lock to kill unit: {}", id);
            let unit_locked = &mut *match unit.lock() {
                Ok(lock) => lock,
                Err(err) => err.into_inner(),
            };
            shutdown_unit(unit_locked, run_info.clone());
        }
        trace!("Killed all units");

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

        #[cfg(feature = "cgroups")]
        {
            let _ = crate::platform::cgroups::move_out_of_own_cgroup(&std::path::PathBuf::from(
                "/sys/fs/cgroup/unified",
            ))
            .map_err(|e| error!("Error while cleaning up cgroups: {}", e));
        }

        println!("Shutdown finished");
        std::process::exit(0);
    });
}
