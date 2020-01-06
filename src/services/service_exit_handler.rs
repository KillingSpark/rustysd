use crate::platform::EventFd;
use crate::signal_handler::ChildTermination;
use crate::units::*;
use std::sync::Arc;

pub fn service_exit_handler(
    pid: nix::unistd::Pid,
    code: ChildTermination,
    run_info: ArcRuntimeInfo,
    notification_socket_path: std::path::PathBuf,
    eventfds: &[EventFd],
) -> Result<(), String> {
    trace!("Exit handler with pid: {}", pid);
    let srvc_id = {
        let unit_table_locked = run_info.unit_table.read().unwrap();
        let entry = {
            let pid_table_locked = &mut *run_info.pid_table.lock().unwrap();
            pid_table_locked.get(&pid).map(|x| {
                let y: PidEntry = *x;
                y
            })
        };
        match entry {
            Some(entry) => match entry {
                PidEntry::Service(id) => id,
                PidEntry::Stop(id) => {
                    trace!(
                        "Stop process for service: {} exited with: {:?}",
                        unit_table_locked
                            .get(&id)
                            .unwrap()
                            .lock()
                            .unwrap()
                            .conf
                            .name(),
                        code
                    );
                    let pid_table_locked = &mut *run_info.pid_table.lock().unwrap();
                    pid_table_locked.remove(&pid);
                    return Ok(());
                }
                PidEntry::PreStart(id) => {
                    trace!(
                        "PreStart process for service: {} exited with: {:?}",
                        unit_table_locked
                            .get(&id)
                            .unwrap()
                            .lock()
                            .unwrap()
                            .conf
                            .name(),
                        code
                    );
                    let pid_table_locked = &mut *run_info.pid_table.lock().unwrap();
                    pid_table_locked.remove(&pid);
                    return Ok(());
                }
                PidEntry::PostStart(id) => {
                    trace!(
                        "PostStart process for service: {} exited with: {:?}",
                        unit_table_locked
                            .get(&id)
                            .unwrap()
                            .lock()
                            .unwrap()
                            .conf
                            .name(),
                        code
                    );
                    let pid_table_locked = &mut *run_info.pid_table.lock().unwrap();
                    pid_table_locked.remove(&pid);
                    return Ok(());
                }
            },
            None => {
                warn!("All spawned processes should have a pid entry");
                return Ok(());
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

            if let Some(conf) = &srvc.service_config {
                if conf.restart == ServiceRestart::Always {
                    let sockets = srvc.socket_names.clone();
                    (name, sockets, true)
                } else {
                    (name, Vec::new(), false)
                }
            } else {
                (name, Vec::new(), false)
            }
        } else {
            (name, Vec::new(), false)
        }
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
        )?;
    } else {
        let unit_locked = unit.lock().unwrap();
        trace!(
            "Killing all services requiring service {}: {:?}",
            name,
            unit_locked.install.required_by
        );
        crate::units::deactivate_units(unit_locked.install.required_by.clone(), run_info.clone());
    }
    Ok(())
}
