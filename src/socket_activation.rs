//! Wait for sockets to activate their respective services

use crate::platform::EventFd;
use crate::units::*;

pub fn start_socketactivation_thread(
    run_info: ArcMutRuntimeInfo,
    note_sock_path: std::path::PathBuf,
    eventfd: crate::platform::EventFd,
    eventfds: std::sync::Arc<Vec<crate::platform::EventFd>>,
) {
    std::thread::spawn(move || loop {
        let wait_result = wait_for_socket(eventfd, run_info.clone());
        match wait_result {
            Ok(ids) => {
                let run_info = run_info.read().unwrap();
                let unit_table = run_info.unit_table;
                for socket_id in ids {
                    {
                        let mut srvc_unit_id = None;
                        for unit in unit_table.values() {
                            if let crate::units::Specific::Service(specific) = &unit.specific {
                                if specific.has_socket(&socket_id.name) {
                                    srvc_unit_id = Some(unit.id.clone());
                                    trace!("Start service {} by socket activation", unit.id.name);
                                }
                            }
                        }

                        if let Some(srvc_unit_id) = srvc_unit_id {
                            let srvc_unit = unit_table.get(&srvc_unit_id).unwrap();
                            let srvc_status = {
                                let status_locked = &*srvc_unit.common.status.read().unwrap();
                                status_locked.clone()
                            };

                            if srvc_status != crate::units::UnitStatus::StartedWaitingForSocket {
                                trace!(
                                    "Ignore socket activation. Service has status: {:?}",
                                    srvc_status
                                );
                                let sock_unit = unit_table.get(&socket_id).unwrap();
                                if let crate::units::Specific::Socket(specific) =
                                    &mut sock_unit.specific
                                {
                                    let mut_state = &mut *specific.state.write().unwrap();
                                    mut_state.sock.activated = true;
                                }
                            } else {
                                match crate::units::activate_unit(
                                    srvc_unit_id,
                                    &*run_info,
                                    note_sock_path.clone(),
                                    eventfds.clone(),
                                    false,
                                ) {
                                    Ok(_) => {
                                        let sock_unit = unit_table.get(&socket_id).unwrap();
                                        if let crate::units::Specific::Socket(specific) =
                                            &mut sock_unit.specific
                                        {
                                            let mut_state = &mut *specific.state.write().unwrap();
                                            mut_state.sock.activated = true;
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
            Err(e) => {
                error!("Error in socket activation loop: {}", e);
                break;
            }
        }
    });
}

pub fn wait_for_socket(
    eventfd: EventFd,
    run_info: ArcMutRuntimeInfo,
) -> Result<Vec<UnitId>, String> {
    let (mut fdset, fd_to_sock_id) = {
        let run_info_locked = &*run_info.read().unwrap();

        let fd_to_sock_id = run_info_locked.fd_store.read().unwrap().global_fds_to_ids();
        let mut fdset = nix::sys::select::FdSet::new();
        {
            let unit_table_locked = &run_info_locked.unit_table;
            for (fd, id) in &fd_to_sock_id {
                let unit = unit_table_locked.get(id).unwrap();
                if let Specific::Socket(specific) = &unit.specific {
                    let mut_state = &*specific.state.read().unwrap();
                    if !mut_state.sock.activated {
                        fdset.insert(*fd);
                    }
                }
            }
            fdset.insert(eventfd.read_end());
        }
        (fdset, fd_to_sock_id)
    };

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
