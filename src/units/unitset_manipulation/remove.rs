use log::trace;

use crate::runtime_info::*;
use crate::units::*;

/// Remove this unit from the run_info and cleanup all references to it
pub fn remove_unit_with_dependencies(
    remove_id: UnitId,
    run_info: &mut RuntimeInfo,
) -> Result<(), String> {
    check_deactivated_recursive(remove_id.clone(), run_info)?;

    let mut depending_by_name_ids = Vec::new();
    {
        find_all_depending(
            remove_id.clone(),
            &run_info.unit_table,
            &mut depending_by_name_ids,
        );
    }
    for id in depending_by_name_ids {
        check_deactivated_recursive(id, run_info)?;
    }

    remove_with_depending_units(remove_id.clone(), &mut run_info.unit_table);

    Ok(())
}

/// Check that this and all units that "require" this unit are stopped
fn check_deactivated_recursive(remove_id: UnitId, run_info: &RuntimeInfo) -> Result<(), String> {
    let unit = run_info.unit_table.get(&remove_id).unwrap();
    let status_locked = unit.common.status.read().unwrap();

    // If the unit is not stopped, return the name of the unit and stop the recursion
    if !status_locked.is_stopped() {
        Err(format!(
            "This unit wasn't stopped before removing: {}",
            unit.id.name
        ))
    } else {
        let next_units = {
            let unit = run_info.unit_table.get(&remove_id).unwrap();
            unit.common.dependencies.required_by.clone()
        };
        for next_id in next_units {
            check_deactivated_recursive(next_id, run_info)?;
        }
        Ok(())
    }
}

/// Remove all occurences of this ID in other units.
/// This requires that this unit is removed at the same time
/// as all units that mention this unit by name!
fn remove_single_unit(rm_id: UnitId, unit_table: &mut UnitTable) {
    for unit in unit_table.values_mut() {
        unit.common.dependencies.remove_id(&rm_id);
    }
    // actuallyy remove the unit from the unit table
    unit_table.remove(&rm_id);
}

fn find_all_depending(rm_id: UnitId, unit_table: &UnitTable, ids: &mut Vec<UnitId>) {
    if ids.contains(&rm_id) {
        return;
    }

    let mut new_ids = Vec::new();
    for (id, unit) in unit_table.iter() {
        if *id != rm_id {
            if unit.common.unit.refs_by_name.contains(&rm_id) {
                new_ids.push(id.clone());
            }
        }
    }

    ids.extend(new_ids.iter().cloned());

    for id in new_ids {
        find_all_depending(id, unit_table, ids);
    }
}

/// Remove all occurences in other units and all units that explicitly mention this unit in their config
fn remove_with_depending_units(rm_id: UnitId, unit_table: &mut UnitTable) {
    trace!("Remove unit: {:?}", rm_id);
    // follow the units install section and check if the units have this unit in their Install-/Unit-config.
    // If so, remove them too

    remove_single_unit(rm_id.clone(), unit_table);
    // first remove all depending units
    let mut next_ids = Vec::new();
    for (id, unit) in unit_table.iter() {
        if *id != rm_id {
            if unit.common.unit.refs_by_name.contains(&rm_id) {
                next_ids.push(id.clone());
            }
        }
    }

    for id in next_ids {
        remove_with_depending_units(id, unit_table);
    }
}
