use crate::runtime_info::*;
use crate::units::*;

pub fn deactivate_unit_recursive(
    id_to_kill: UnitId,
    run_info: &RuntimeInfo,
) -> Result<(), UnitOperationError> {
    let unit = match run_info.unit_table.get(&id_to_kill) {
        Some(unit) => unit,
        None => {
            // If this occurs, there is a flaw in the handling of dependencies
            // IDs should be purged globally when units get removed
            return Err(UnitOperationError {
                reason: UnitOperationErrorReason::GenericStartError(
                    "Tried to activate a unit that can not be found".into(),
                ),
                unit_name: id_to_kill.name.clone(),
                unit_id: id_to_kill.clone(),
            });
        }
    };
    let kill_before_this = unit.common.dependencies.required_by.clone();

    deactivate_units_recursive(kill_before_this, run_info)?;

    deactivate_unit_checkdeps(id_to_kill, run_info.clone())
}

pub fn deactivate_unit_checkdeps(
    id_to_kill: UnitId,
    run_info: &RuntimeInfo,
) -> Result<(), UnitOperationError> {
    let unit = match run_info.unit_table.get(&id_to_kill) {
        Some(unit) => unit,
        None => {
            // If this occurs, there is a flaw in the handling of dependencies
            // IDs should be purged globally when units get removed
            return Err(UnitOperationError {
                reason: UnitOperationErrorReason::GenericStartError(
                    "Tried to activate a unit that can not be found".into(),
                ),
                unit_name: id_to_kill.name.clone(),
                unit_id: id_to_kill.clone(),
            });
        }
    };
    let unkilled_depending =
        unit.common
            .dependencies
            .kill_before_this()
            .iter()
            .fold(Vec::new(), |mut acc, elem| {
                let elem_unit = run_info.unit_table.get(elem).unwrap();
                let status_locked = elem_unit.common.status.read().unwrap();

                if status_locked.is_started() {
                    acc.push(elem.clone());
                }
                acc
            });
    if !unkilled_depending.is_empty() {
        trace!(
            "Unit: {} ignores deactivation. Not all units depending on this unit have been started (still waiting for: {:?})",
            unit.id.name,
            unkilled_depending,
        );
        return Err(UnitOperationError {
            reason: UnitOperationErrorReason::DependencyError(unkilled_depending),
            unit_name: unit.id.name.clone(),
            unit_id: unit.id.clone(),
        });
    }

    deactivate_unit(id_to_kill, run_info)
}

pub fn deactivate_unit(
    id_to_kill: UnitId,
    run_info: &RuntimeInfo,
) -> Result<(), UnitOperationError> {
    let unit = match run_info.unit_table.get(&id_to_kill) {
        Some(unit) => unit,
        None => {
            // If this occurs, there is a flaw in the handling of dependencies
            // IDs should be purged globally when units get removed
            return Err(UnitOperationError {
                reason: UnitOperationErrorReason::GenericStartError(
                    "Tried to activate a unit that can not be found".into(),
                ),
                unit_name: id_to_kill.name.clone(),
                unit_id: id_to_kill.clone(),
            });
        }
    };
    unit.deactivate(run_info.clone())?;

    Ok(())
}

pub fn deactivate_units_recursive(
    ids_to_kill: Vec<UnitId>,
    run_info: &RuntimeInfo,
) -> Result<(), UnitOperationError> {
    for id in ids_to_kill {
        deactivate_unit_recursive(id, run_info)?;
    }
    Ok(())
}

pub fn deactivate_units(
    ids_to_kill: Vec<UnitId>,
    run_info: &RuntimeInfo,
) -> Result<(), UnitOperationError> {
    for id in ids_to_kill {
        deactivate_unit_checkdeps(id, run_info.clone())?;
    }
    Ok(())
}

pub fn reactivate_unit_checkdeps(
    id_to_restart: UnitId,
    run_info: &RuntimeInfo,
) -> std::result::Result<(), UnitOperationError> {
    trace!("Reactivation of unit: {:?}. Deactivate", id_to_restart);
    let unit = match run_info.unit_table.get(&id_to_restart) {
        Some(unit) => unit,
        None => {
            // If this occurs, there is a flaw in the handling of dependencies
            // IDs should be purged globally when units get removed
            return Err(UnitOperationError {
                reason: UnitOperationErrorReason::GenericStartError(
                    "Tried to activate a unit that can not be found".into(),
                ),
                unit_name: id_to_restart.name.clone(),
                unit_id: id_to_restart.clone(),
            });
        }
    };
    // if not all dependencies are yet started ignore this call
    let unstarted_deps = unstarted_deps(&id_to_restart, run_info);
    if !unstarted_deps.is_empty() {
        trace!(
            "Unit: {} ignores activation. Not all dependencies have been started (still waiting for: {:?})",
            unit.id.name,
            unstarted_deps,
        );
        return Err(UnitOperationError {
            reason: UnitOperationErrorReason::DependencyError(unstarted_deps),
            unit_name: unit.id.name.clone(),
            unit_id: unit.id.clone(),
        });
    }
    reactivate_unit(id_to_restart, run_info)
}

pub fn reactivate_unit(
    id_to_restart: UnitId,
    run_info: &RuntimeInfo,
) -> std::result::Result<(), UnitOperationError> {
    trace!("Reactivation of unit: {:?}. Deactivate", id_to_restart);
    let unit = match run_info.unit_table.get(&id_to_restart) {
        Some(unit) => unit,
        None => {
            // If this occurs, there is a flaw in the handling of dependencies
            // IDs should be purged globally when units get removed
            return Err(UnitOperationError {
                reason: UnitOperationErrorReason::GenericStartError(
                    "Tried to activate a unit that can not be found".into(),
                ),
                unit_name: id_to_restart.name.clone(),
                unit_id: id_to_restart.clone(),
            });
        }
    };
    unit.reactivate(run_info, crate::units::ActivationSource::Regular)
}
