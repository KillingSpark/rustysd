use crate::units::*;


/// Remove this unit from the run_info and cleanup all references to it
pub fn remove_unit_with_dependencies(
    remove_id: UnitId,
    run_info: &ArcRuntimeInfo,
) -> Result<(), String> {
    check_deactivated_recursive(remove_id, run_info)?;

    let unit_table_locked = &mut *run_info.unit_table.write().unwrap();
    remove_with_depending_units(remove_id, unit_table_locked);

    Ok(())
}

/// Check that this and all units that "require" this unit are stopped
fn check_deactivated_recursive(remove_id: UnitId, run_info: &ArcRuntimeInfo) -> Result<(), String> {
    let status_table = &*run_info.status_table.read().unwrap();
    let status = status_table.get(&remove_id).unwrap();
    let status_locked = &*status.lock().unwrap();

    // If the unit is not stopped, return the name of the unit and stop the recursion
    if !status_locked.is_stopped() {
        let unit_table_locked = &*run_info.unit_table.read().unwrap();
        let unit = unit_table_locked.get(&remove_id).unwrap();
        let name = unit.lock().unwrap().conf.name();
        Err(format!(
            "This unit wasn't stopped before removing: {}",
            name
        ))
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

/// Remove all occurences of this id from the vec
fn remove_id(id: &UnitId, ids: &mut Vec<UnitId>) {
    while let Some(idx) = ids.iter().position(|e| *e == *id) {
        ids.remove(idx);
    }
}

// Remove all occurences of this ID in other units
fn remove_single_unit(rm_id: UnitId, unit_table: &mut UnitTable) {
    // follow the units install section and remove this units Id
    // TODO Might be easier / less prone to errors to just do this for all units, even if it's unecessary?
    let install = {
        let unit = unit_table.get(&rm_id).unwrap();
        let unit_locked = unit.lock().unwrap();
        unit_locked.install.clone()
    };

    for x in &install.before {
        // remove the before <-> after
        let unit = unit_table.get(&rm_id).unwrap();
        let unit_locked = &mut *unit.lock().unwrap();
        remove_id(x, &mut unit_locked.install.after);
    }
    for x in &install.after {
        // remove the after <-> before
        let unit = unit_table.get(&rm_id).unwrap();
        let unit_locked = &mut *unit.lock().unwrap();
        remove_id(x, &mut unit_locked.install.before);
    }
    for x in &install.requires {
        // remove the required <-> required_by
        let unit = unit_table.get(&rm_id).unwrap();
        let unit_locked = &mut *unit.lock().unwrap();
        remove_id(x, &mut unit_locked.install.required_by);
    }
    for x in &install.required_by {
        // remove the required_by <-> required
        let unit = unit_table.get(&rm_id).unwrap();
        let unit_locked = &mut *unit.lock().unwrap();
        remove_id(x, &mut unit_locked.install.requires);
    }
    for x in &install.wants {
        // remove the wants <-> wanted_by
        let unit = unit_table.get(&rm_id).unwrap();
        let unit_locked = &mut *unit.lock().unwrap();
        remove_id(x, &mut unit_locked.install.wanted_by);
    }
    for x in &install.wanted_by {
        // remove the wanted_by <-> wants
        let unit = unit_table.get(&rm_id).unwrap();
        let unit_locked = &mut *unit.lock().unwrap();
        remove_id(x, &mut unit_locked.install.wants);
    }

    // actuallyy remove the unit from the unit table
    unit_table.remove(&rm_id);
}

/// Remove all occurences in other units and all units that explicitly mention this unit in their config
fn remove_with_depending_units(rm_id: UnitId, unit_table: &mut UnitTable) {
    // follow the units install section and check if the units have this unit in their Install-/Unit-config.
    // If so, remove them too

    let rm_name = {
        let unit = unit_table.get(&rm_id).unwrap();
        unit.lock().unwrap().conf.name()
    };

    let mut names = Vec::new();
    let mut next_ids = Vec::new();
    for (id, unit) in unit_table.iter() {
        if *id != rm_id {
            let unit = unit_table.get(&rm_id).unwrap();
            let unit_locked = &mut *unit.lock().unwrap();
            names.clear();
            collect_names_needed(unit_locked, &mut names);
            if names.contains(&rm_name) {
                next_ids.push(*id)
            }
        }
    }

    for id in next_ids {
        remove_with_depending_units(id, unit_table);
    }

    // after all depending units have been removed, this can be called
    remove_single_unit(rm_id, unit_table);
}
