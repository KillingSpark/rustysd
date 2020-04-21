use crate::units::*;

pub fn deactivate_unit_recursive(
    id_to_kill: UnitId,
    run_info: &RuntimeInfo,
) -> Result<(), UnitOperationError> {
    let kill_before_this = {
        let unit = run_info.unit_table.get(&id_to_kill).unwrap();
        unit.common.dependencies.required_by.clone()
    };
    deactivate_units_recursive(kill_before_this, run_info)?;

    deactivate_unit(id_to_kill, run_info.clone())
}
pub fn deactivate_unit(
    id_to_kill: UnitId,
    run_info: &RuntimeInfo,
) -> Result<(), UnitOperationError> {
    // TODO deal with kill final
    let unit = run_info.unit_table.get(&id_to_kill).unwrap();
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
        deactivate_unit(id, run_info.clone())?;
    }
    Ok(())
}

pub fn reactivate_unit(
    id_to_restart: UnitId,
    run_info: &RuntimeInfo,
) -> std::result::Result<(), UnitOperationError> {
    trace!("Reactivation of unit: {:?}. Deactivate", id_to_restart);
    deactivate_unit(id_to_restart.clone(), run_info.clone())?;
    trace!(
        "Reactivation of unit: {:?}. Deactivation ran. Activate again",
        id_to_restart
    );
    crate::units::activate_unit(id_to_restart.clone(), run_info, ActivationSource::Regular)
        .map(|_| ())
}
