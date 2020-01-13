//! Wait for sockets to activate their respective services

use crate::platform::EventFd;
use crate::units::*;

pub fn start_socketactivation_thread(
    run_info: ArcRuntimeInfo,
    note_sock_path: std::path::PathBuf,
    eventfd: crate::platform::EventFd,
    eventfds: std::sync::Arc<Vec<crate::platform::EventFd>>,
) {
    std::thread::spawn(move || loop {
        match wait_for_socket(
            eventfd,
            run_info.unit_table.clone(),
            run_info.fd_store.clone(),
        ) {
            Ok(ids) => {
                for socket_id in ids {
                    let unit_table_locked = run_info.unit_table.read().unwrap();
                    {
                        let socket_name = {
                            let sock_unit = unit_table_locked.get(&socket_id).unwrap();
                            let sock_unit_locked = sock_unit.lock().unwrap();
                            sock_unit_locked.conf.name()
                        };

                        let mut srvc_unit_id = None;
                        for unit in unit_table_locked.values() {
                            let unit_locked = unit.lock().unwrap();
                            if let crate::units::UnitSpecialized::Service(srvc) =
                                &unit_locked.specialized
                            {
                                if srvc.socket_names.contains(&socket_name) {
                                    srvc_unit_id = Some(unit_locked.id);
                                    trace!(
                                        "Start service {} by socket activation",
                                        unit_locked.conf.name()
                                    );
                                }
                            }
                        }

                        if let Some(srvc_unit_id) = srvc_unit_id {
                            if let Some(status) =
                                run_info.status_table.read().unwrap().get(&srvc_unit_id)
                            {
                                let srvc_status = {
                                    let status_locked = status.lock().unwrap();
                                    status_locked.clone()
                                };

                                if srvc_status != crate::units::UnitStatus::StartedWaitingForSocket
                                {
                                    trace!(
                                        "Ignore socket activation. Service has status: {:?}",
                                        srvc_status
                                    );
                                    let sock_unit = unit_table_locked.get(&socket_id).unwrap();
                                    let mut sock_unit_locked = sock_unit.lock().unwrap();
                                    if let crate::units::UnitSpecialized::Socket(sock) =
                                        &mut sock_unit_locked.specialized
                                    {
                                        sock.activated = true;
                                    }
                                } else {
                                    match crate::units::activate_unit(
                                        srvc_unit_id,
                                        run_info.clone(),
                                        note_sock_path.clone(),
                                        eventfds.clone(),
                                        false,
                                    ) {
                                        Ok(_) => {
                                            let sock_unit =
                                                unit_table_locked.get(&socket_id).unwrap();
                                            let mut sock_unit_locked = sock_unit.lock().unwrap();
                                            if let crate::units::UnitSpecialized::Socket(sock) =
                                                &mut sock_unit_locked.specialized
                                            {
                                                sock.activated = true;
                                            }
                                        }
                                        Err(e) => {
                                            format!(
                                                "Error while starting service from socket activation: {}",
                                                e
                                            );
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                error!("Error in socket activation loop: {}", e);
                break;
            }
        }
    });
}

pub fn wait_for_socket(
    eventfd: EventFd,
    unit_table: ArcMutUnitTable,
    fd_store: ArcMutFDStore,
) -> Result<Vec<UnitId>, String> {
    let fd_to_sock_id = fd_store.read().unwrap().global_fds_to_ids();

    let mut fdset = nix::sys::select::FdSet::new();
    {
        let unit_table_locked = unit_table.read().unwrap();
        for (fd, id) in &fd_to_sock_id {
            let unit = unit_table_locked.get(id).unwrap();
            let unit_locked = unit.lock().unwrap();
            if let UnitSpecialized::Socket(sock) = &unit_locked.specialized {
                if !sock.activated {
                    fdset.insert(*fd);
                }
            }
        }
        fdset.insert(eventfd.read_end());
    }

    let result = nix::sys::select::select(None, Some(&mut fdset), None, None, None);
    match result {
        Ok(_) => {
            let mut activated_ids = Vec::new();
            if fdset.contains(eventfd.read_end()) {
                trace!("Interrupted socketactivation select because the eventfd fired");
                crate::platform::reset_event_fd(eventfd);
                trace!("Reset eventfd value");
            } else {
                for (fd, id) in &fd_to_sock_id {
                    if fdset.contains(*fd) {
                        activated_ids.push(*id);
                    }
                }
            }
            Ok(activated_ids)
        }
        Err(e) => {
            if let nix::Error::Sys(nix::errno::Errno::EINTR) = e {
                Ok(Vec::new())
            } else {
                Err(format!("Error while selecting: {}", e))
            }
        }
    }
}
