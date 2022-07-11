use log::trace;

use crate::runtime_info::*;
use crate::units::*;

pub fn deactivate_unit_recursive(
    id_to_kill: &UnitId,
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

    deactivate_units_recursive(&unit.common.dependencies.required_by, run_info)?;

    deactivate_unit(id_to_kill, run_info.clone())
}

pub fn deactivate_unit(
    id_to_kill: &UnitId,
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
    ids_to_kill: &[UnitId],
    run_info: &RuntimeInfo,
) -> Result<(), UnitOperationError> {
    for id in ids_to_kill {
        deactivate_unit_recursive(id, run_info)?;
    }
    Ok(())
}

pub fn deactivate_units(
    ids_to_kill: &[UnitId],
    run_info: &RuntimeInfo,
) -> Result<(), UnitOperationError> {
    for id in ids_to_kill {
        deactivate_unit(id, run_info.clone())?;
    }
    Ok(())
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
