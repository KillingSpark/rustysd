use crate::platform::EventFd;
use crate::units::*;
use std::sync::Arc;

pub fn deactivate_unit_recursive(id_to_kill: UnitId, killfinal: bool, run_info: ArcRuntimeInfo) {
    let kill_before_this = {
        let unit = {
            let unit_table_locked = run_info.unit_table.read().unwrap();
            unit_table_locked.get(&id_to_kill).unwrap().clone()
        };
        let unit_locked = &mut *unit.lock().unwrap();
        unit_locked.install.required_by.clone()
    };

    deactivate_units_recursive(kill_before_this, killfinal, run_info.clone());

    deactivate_unit(id_to_kill, killfinal, run_info.clone());
}
pub fn deactivate_unit(id_to_kill: UnitId, killfinal: bool, run_info: ArcRuntimeInfo) {
    let unit = {
        let unit_table_locked = run_info.unit_table.read().unwrap();
        unit_table_locked.get(&id_to_kill).unwrap().clone()
    };
    let unit_locked = &mut *unit.lock().unwrap();

    {
        let status_table_locked = run_info.status_table.read().unwrap();
        let status = status_table_locked.get(&id_to_kill).unwrap();
        let status_locked = &mut *status.lock().unwrap();
        match *status_locked {
            UnitStatus::Started | UnitStatus::StartedWaitingForSocket | UnitStatus::Starting => {
                *status_locked = UnitStatus::Stopping;
            }
            UnitStatus::NeverStarted
            | UnitStatus::Stopped
            | UnitStatus::StoppedFinal
            | UnitStatus::Stopping => {
                return;
            }
        }
    }
    unit_locked
        .deactivate(run_info.pid_table.clone(), run_info.fd_store.clone())
        .unwrap();
    {
        let status_table_locked = run_info.status_table.read().unwrap();
        let status = status_table_locked.get(&id_to_kill).unwrap();
        let mut status_locked = status.lock().unwrap();
        if killfinal {
            *status_locked = UnitStatus::StoppedFinal;
        }else{
            *status_locked = UnitStatus::Stopped;
        }
    }
}

pub fn deactivate_units_recursive(ids_to_kill: Vec<UnitId>, killfinal: bool, run_info: ArcRuntimeInfo) {
    for id in ids_to_kill {
        deactivate_unit_recursive(id, killfinal, run_info.clone());
    }
}

pub fn deactivate_units(ids_to_kill: Vec<UnitId>, killfinal: bool, run_info: ArcRuntimeInfo) {
    //TODO deactivate all units that require these unit
    for id in ids_to_kill {
        deactivate_unit(id, killfinal, run_info.clone());
    }
}

pub fn reactivate_unit(
    id_to_restart: UnitId,
    run_info: ArcRuntimeInfo,
    notification_socket_path: std::path::PathBuf,
    eventfds: Arc<Vec<EventFd>>,
) -> std::result::Result<(), std::string::String> {
    deactivate_unit(id_to_restart, false, run_info.clone());
    crate::units::activate_unit(
        id_to_restart,
        run_info,
        notification_socket_path,
        eventfds,
        true,
    )
    .map(|_| ())
}
