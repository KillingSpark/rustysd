//! Wait for sockets to activate their respective services
use log::error;
use log::trace;

use crate::runtime_info::*;
use crate::units::*;

pub fn start_socketactivation_thread(run_info: ArcMutRuntimeInfo) {
    std::thread::spawn(move || loop {
        let wait_result = wait_for_socket(run_info.clone());
        match wait_result {
            Ok(ids) => {
                let run_info = run_info.read().unwrap();
                let unit_table = &run_info.unit_table;
                for socket_id in ids {
                    {
                        // search the service this socket belongs to.
                        // Note that this differs from systemd behaviour where one socket may belong to multiple services
                        let mut srvc_unit = None;
                        for unit in unit_table.values() {
                            if let crate::units::Specific::Service(specific) = &unit.specific {
                                if specific.has_socket(&socket_id.name) {
                                    srvc_unit = Some(unit);
                                    trace!("Start service {} by socket activation", unit.id.name);
                                    break;
                                }
                            }
                        }

                        // mark socket as activated, removing it from the set of
                        // fds rustysd is actively listening on
                        let sock_unit = unit_table.get(&socket_id).unwrap();
                        if let Specific::Socket(specific) = &sock_unit.specific {
                            let mut_state = &mut *specific.state.write().unwrap();
                            mut_state.sock.activated = true;
                        }
                        if srvc_unit.is_none() {
                            error!(
                                "Socket unit {:?} activated, but the service could not be found",
                                socket_id
                            );
                        }
                        if let Some(srvc_unit) = srvc_unit {
                            let srvc_status = {
                                let status_locked = &*srvc_unit.common.status.read().unwrap();
                                status_locked.clone()
                            };

                            if srvc_status != UnitStatus::Started(StatusStarted::WaitingForSocket) {
                                // This should not happen too often because the sockets of a service
                                // should only be listened on if the service is currently waiting on socket activation
                                trace!(
                                    "Ignore socket activation. Service has status: {:?}",
                                    srvc_status
                                );
                            } else {
                                // the service unit gets activated
                                match crate::units::activate_unit(
                                    srvc_unit.id.clone(),
                                    &*run_info,
                                    ActivationSource::SocketActivation,
                                ) {
                                    Ok(_) => {
                                        trace!(
                                            "New status after socket activation: {:?}",
                                            *unit_table
                                                .get(&srvc_unit.id)
                                                .unwrap()
                                                .common
                                                .status
                                                .read()
                                                .unwrap()
                                        );
                                    }
                                    Err(e) => {
                                        format!(
                                                "Error while starting service from socket activation: {}",
                                                e
                                            );
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

pub fn wait_for_socket(run_info: ArcMutRuntimeInfo) -> Result<Vec<UnitId>, String> {
    let eventfd = { run_info.read().unwrap().socket_activation_eventfd };
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
                        activated_ids.push(id.clone());
                    }
                }
            }
            Ok(activated_ids)
        }
        Err(e) => {
            if let nix::Error::EINTR = e {
                Ok(Vec::new())
            } else {
                Err(format!("Error while selecting: {}", e))
            }
        }
    }
}
