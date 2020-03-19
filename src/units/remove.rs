use crate::units::*;

pub fn remove_unit_with_dependencies(
    remove_id: UnitId,
    run_info: &ArcRuntimeInfo,
) -> Result<(), String> {
    check_deactivated_recursive(remove_id, run_info)?;

    let unit_table_locked = &mut *run_info.unit_table.write().unwrap();
    remove_with_depending_units(remove_id, unit_table_locked);

    Ok(())
}

fn check_deactivated_recursive(remove_id: UnitId, run_info: &ArcRuntimeInfo) -> Result<(), String> {
    let status_table = &*run_info.status_table.read().unwrap();
    let status = status_table.get(&remove_id).unwrap();
    let status_locked = &*status.lock().unwrap();
    if !status_locked.is_stopped() {
        let unit_table_locked = &*run_info.unit_table.read().unwrap();
        let unit = unit_table_locked.get(&remove_id).unwrap();
        let name = unit.lock().unwrap().conf.name();
        Err(name)
    } else {
        let next_units = {
            let unit_table_locked = &*run_info.unit_table.read().unwrap();
            let unit = unit_table_locked.get(&remove_id).unwrap();
            let unit_locked = unit.lock().unwrap();
            unit_locked.install.required_by.clone()
        };
        for next_id in next_units {
            check_deactivated_recursive(next_id, run_info)?;
        }
        Ok(())
    }
}

fn remove_single_unit(remove_id: UnitId, unit_table: &mut UnitTable) {
    // follow the units install section and remove this units Id
    let install = {
        let unit = unit_table.get(&remove_id).unwrap();
        let unit_locked = unit.lock().unwrap();
        unit_locked.install.clone()
    };

    for x in &install.before {
        let unit = unit_table.get(&remove_id).unwrap();
        let unit_locked = unit.lock().unwrap();

        // remove the before <-> after
    }
    for x in &install.after {
        let unit = unit_table.get(&remove_id).unwrap();
        let unit_locked = unit.lock().unwrap();

        // remove the after <-> before
    }
    for x in &install.requires {
        let unit = unit_table.get(&remove_id).unwrap();
        let unit_locked = unit.lock().unwrap();

        // remove the required <-> reuqired_by
    }
    for x in &install.required_by {
        let unit = unit_table.get(&remove_id).unwrap();
        let unit_locked = unit.lock().unwrap();

        // remove the required_by <-> required
    }
    for x in &install.wants {
        let unit = unit_table.get(&remove_id).unwrap();
        let unit_locked = unit.lock().unwrap();

        // remove the wanted <-> wanted_by
    }
    for x in &install.wanted_by {
        let unit = unit_table.get(&remove_id).unwrap();
        let unit_locked = unit.lock().unwrap();

        // remove the wanted_by <-> wanted
    }
}

fn remove_with_depending_units(remove_id: UnitId, unit_table: &mut UnitTable) {
    // follow the units install section and check if the units have this unit in their Install-/Unit-config.
    // If so, remove them too

    // after all depending units have been removed, this can be called
    remove_single_unit(remove_id, unit_table);
}
