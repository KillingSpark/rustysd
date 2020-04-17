use crate::platform::EventFd;
use crate::units::*;
use std::sync::Arc;

pub fn deactivate_unit_recursive(
    id_to_kill: UnitId,
    killfinal: bool,
    run_info: &RuntimeInfo,
) -> Result<(), UnitOperationError> {
    let kill_before_this = {
        let unit = run_info.unit_table.get(&id_to_kill).unwrap();
        unit.common.dependencies.required_by.clone()
    };
    deactivate_units_recursive(kill_before_this, killfinal, run_info)?;

    deactivate_unit(id_to_kill, killfinal, run_info.clone())
}
pub fn deactivate_unit(
    id_to_kill: UnitId,
    killfinal: bool,
    run_info: &RuntimeInfo,
) -> Result<(), UnitOperationError> {
    // TODO deal with kill final
    let unit = run_info.unit_table.get(&id_to_kill).unwrap();
    unit.deactivate(run_info.clone())?;
    Ok(())
}

pub fn deactivate_units_recursive(
    ids_to_kill: Vec<UnitId>,
    killfinal: bool,
    run_info: &RuntimeInfo,
) -> Result<(), UnitOperationError> {
    for id in ids_to_kill {
        deactivate_unit_recursive(id, killfinal, run_info)?;
    }
    Ok(())
}

pub fn deactivate_units(
    ids_to_kill: Vec<UnitId>,
    killfinal: bool,
    run_info: &RuntimeInfo,
) -> Result<(), UnitOperationError> {
    for id in ids_to_kill {
        deactivate_unit(id, killfinal, run_info.clone())?;
    }
    Ok(())
}

pub fn reactivate_unit(
    id_to_restart: UnitId,
    run_info: &RuntimeInfo,
    notification_socket_path: std::path::PathBuf,
    eventfds: Arc<Vec<EventFd>>,
) -> std::result::Result<(), UnitOperationError> {
    deactivate_unit(id_to_restart.clone(), false, run_info.clone())?;
    crate::units::activate_unit(
        id_to_restart.clone(),
        run_info,
        notification_socket_path,
        eventfds,
        true,
    )
    .map(|_| ())
}
