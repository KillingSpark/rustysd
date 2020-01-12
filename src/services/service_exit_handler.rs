use crate::platform::EventFd;
use crate::signal_handler::ChildTermination;
use crate::units::*;
use std::sync::Arc;

pub fn service_exit_handler_new_thread(
    pid: nix::unistd::Pid,
    code: ChildTermination,
    run_info: ArcRuntimeInfo,
    notification_socket_path: std::path::PathBuf,
    eventfds: Vec<EventFd>,
) {
    std::thread::spawn(move || {
        if let Err(e) =
            service_exit_handler(pid, code, run_info, notification_socket_path, &eventfds)
        {
            error!("{}", e);
        }
    });
}

pub fn service_exit_handler(
    pid: nix::unistd::Pid,
    code: ChildTermination,
    run_info: ArcRuntimeInfo,
    notification_socket_path: std::path::PathBuf,
    eventfds: &[EventFd],
) -> Result<(), String> {
    trace!("Exit handler with pid: {}", pid);

    // Handle exiting of helper processes and oneshot processes
    {
        let pid_table_locked = &mut *run_info.pid_table.lock().unwrap();
        let entry = pid_table_locked.get(&pid);
        match entry {
            Some(entry) => match entry {
                PidEntry::Service(_id, srvctype) => {
                    if *srvctype == ServiceType::OneShot {
                        trace!("Save oneshot service as exited. PID: {}", pid);
                        pid_table_locked.insert(pid, PidEntry::OneshotExited(code));
                        return Ok(());
                    }
                }
                PidEntry::Helper(_id, srvc_name) => {
                    trace!(
                        "Helper process for service: {} exited with: {:?}",
                        srvc_name,
                        code
                    );
                    // this will be collected by the thread that waits for the helper process to exit
                    pid_table_locked.insert(pid, PidEntry::HelperExited(code));
                    return Ok(());
                }
                PidEntry::HelperExited(_) => {
                    // TODO is this sensibel? How do we handle this?
                    error!("Pid exited that was already saved as exited");
                    return Ok(());
                }
                PidEntry::OneshotExited(_) => {
                    // TODO is this sensibel? How do we handle this?
                    error!("Pid exited that was already saved as exited");
                    return Ok(());
                }
            },
            None => {
                warn!(
                    "All spawned processes should have a pid entry. This did not: {}",
                    pid
                );
                return Ok(());
            }
        }
    }

    let srvc_id = {
        let pid_table_locked = &mut *run_info.pid_table.lock().unwrap();
        let entry = pid_table_locked.remove(&pid);
        match entry {
            Some(entry) => match entry {
                PidEntry::Service(id, _) => id,
                PidEntry::Helper(_id, _srvc_name) => {
                    unreachable!();
                }
                PidEntry::HelperExited(_) => {
                    unreachable!();
                }
                PidEntry::OneshotExited(_) => {
                    unreachable!();
                }
            },
            None => {
                unreachable!();
            }
        }
    };

    let unit = {
        let unit_table_locked = run_info.unit_table.read().unwrap();
        match unit_table_locked.get(&srvc_id) {
            Some(unit) => Arc::clone(unit),
            None => {
                panic!("Tried to run a unit that has been removed from the map");
            }
        }
    };

    trace!("Check if we want to restart the unit");
    let (name, sockets, restart_unit) = {
        let unit_locked = &mut *unit.lock().unwrap();
        let name = unit_locked.conf.name();
        if let UnitSpecialized::Service(srvc) = &mut unit_locked.specialized {
            trace!(
                "Service with id: {:?}, name: {} pid: {} exited with: {:?}",
                srvc_id,
                unit_locked.conf.name(),
                pid,
                code
            );

            if srvc.service_config.restart == ServiceRestart::Always {
                let sockets = srvc.socket_names.clone();
                (name, sockets, true)
            } else {
                (name, Vec::new(), false)
            }
        } else {
            (name, Vec::new(), false)
        }
    };

    let restart_unit = if restart_unit {
        let status_table_locked = run_info.status_table.read().unwrap();
        let status_locked = &*status_table_locked.get(&srvc_id).unwrap().lock().unwrap();
        // if thats not the case this service got killed by something else so dont interfere
        *status_locked == UnitStatus::Started
    } else {
        false
    };

    if restart_unit {
        {
            // tell socket activation to listen to these sockets again
            for unit in run_info.unit_table.read().unwrap().values() {
                let mut unit_locked = unit.lock().unwrap();
                if sockets.contains(&unit_locked.conf.name()) {
                    if let UnitSpecialized::Socket(sock) = &mut unit_locked.specialized {
                        sock.activated = false;
                    }
                }
            }
        }
        trace!("Restart service {} after it died", name);
        crate::units::reactivate_unit(
            srvc_id,
            run_info,
            notification_socket_path,
            Arc::new(eventfds.to_vec()),
        )
        .map_err(|e| format!("{}", e))?;
    } else {
        trace!(
            "Recursively killing all services requiring service {}",
            name
        );
        crate::units::deactivate_unit_recursive(srvc_id, true, run_info.clone());
    }
    Ok(())
}
