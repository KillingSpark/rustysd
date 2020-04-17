use crate::platform::EventFd;
use crate::signal_handler::ChildTermination;
use crate::units::*;
use std::sync::Arc;

pub fn service_exit_handler_new_thread(
    pid: nix::unistd::Pid,
    code: ChildTermination,
    run_info: ArcMutRuntimeInfo,
    notification_socket_path: std::path::PathBuf,
    eventfds: Vec<EventFd>,
) {
    std::thread::spawn(move || {
        if let Err(e) = service_exit_handler(
            pid,
            code,
            &*run_info.read().unwrap(),
            notification_socket_path,
            &eventfds,
        ) {
            error!("{}", e);
        }
    });
}

pub fn service_exit_handler(
    pid: nix::unistd::Pid,
    code: ChildTermination,
    run_info: &RuntimeInfo,
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
                PidEntry::Service(_id, _srvctype) => {
                    // ignore at this point, will be handled below
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
                trace!(
                    "All processes spawned by rustysd have a pid entry. This did not: {}. Probably a rerooted orphan that got killed.",
                    pid
                );
                return Ok(());
            }
        }
    }

    // find out which service exited and if it was a oneshot service save an entry in the pid table that marks the service as exited
    let srvc_id = {
        let pid_table_locked = &mut *run_info.pid_table.lock().unwrap();
        let entry = pid_table_locked.remove(&pid);
        match entry {
            Some(entry) => match entry {
                PidEntry::Service(id, srvctype) => {
                    if srvctype == ServiceType::OneShot {
                        trace!("Save oneshot service as exited. PID: {}", pid);
                        pid_table_locked.insert(pid, PidEntry::OneshotExited(code));
                    }
                    id
                }
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

    let unit = match run_info.unit_table.get(&srvc_id) {
        Some(unit) => unit,
        None => {
            panic!("Tried to run a unit that has been removed from the map");
        }
    };

    // kill oneshot service processes. There should be none but just in case...
    {
        if let Specific::Service(srvc) = &unit.specific {
            if srvc.conf.srcv_type == ServiceType::OneShot {
                let mut_state = &mut *srvc.state.write().unwrap();
                mut_state.srvc.kill_all_remaining_processes(&unit.id.name);
                return Ok(());
            }
        }
    }

    trace!("Check if we want to restart the unit");
    let (name, sockets, restart_unit) = {
        let name = &unit.id.name;
        if let Specific::Service(srvc) = &unit.specific {
            trace!(
                "Service with id: {:?}, name: {} pid: {} exited with: {:?}",
                srvc_id,
                unit.id.name,
                pid,
                code
            );

            if srvc.conf.restart == ServiceRestart::Always {
                let sockets = srvc.conf.sockets.clone();
                (name, sockets, true)
            } else {
                (name, Vec::new(), false)
            }
        } else {
            (name, Vec::new(), false)
        }
    };

    // check that the status is "Started". If thats not the case this service got killed by something else (control interface for example) so dont interfere
    {
        let status_locked = &*unit.common.status.read().unwrap();
        if *status_locked != UnitStatus::Started && *status_locked != UnitStatus::Starting {
            trace!("Exit handler ignores exit of service {}. Its status is not 'Started/Starting', it is: {:?}", name, *status_locked);
            return Ok(());
        }
    }

    if restart_unit {
        {
            // tell socket activation to listen to these sockets again
            for unit in run_info.unit_table.values() {
                if sockets.contains(&unit.id.name) {
                    if let Specific::Socket(sock) = &unit.specific {
                        let mut_state = &mut *sock.state.write().unwrap();
                        mut_state.sock.activated = false;
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
        crate::units::deactivate_unit_recursive(srvc_id, true, run_info.clone())
            .map_err(|e| format!("{}", e))?;
    }
    Ok(())
}
