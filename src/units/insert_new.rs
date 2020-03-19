use crate::units;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

fn find_new_unit_path(unit_dirs: &[PathBuf], find_name: &str) -> Result<Option<PathBuf>, String> {
    for dir in unit_dirs {
        for entry in
            fs::read_dir(dir).map_err(|e| format!("Error while opening dir {:?}: {}", dir, e))?
        {
            let entry = entry.unwrap();
            let meta = entry.metadata().unwrap();
            if meta.file_type().is_file() {
                if entry.file_name() == find_name {
                    return Ok(Some(entry.path()));
                }
            }
            if meta.file_type().is_dir() {
                if let Some(p) = find_new_unit_path(&[entry.path()], find_name)? {
                    return Ok(Some(p));
                }
            }
        }
    }

    Ok(None)
}

/// Loads a unit with a given name. It searches all pathes recursively until it finds a file with a matching name
pub fn load_new_unit(
    unit_dirs: &[PathBuf],
    find_name: &str,
    next_id: u64,
) -> Result<units::Unit, String> {
    if let Some(unit_path) = find_new_unit_path(unit_dirs, find_name)? {
        let content = fs::read_to_string(&unit_path).map_err(|e| {
            format!(
                "{}",
                units::ParsingError::new(
                    units::ParsingErrorReason::from(Box::new(e)),
                    unit_path.clone()
                )
            )
        })?;
        let parsed = units::parse_file(&content)
            .map_err(|e| format!("{}", units::ParsingError::new(e, unit_path.clone())))?;
        let unit = if find_name.ends_with(".service") {
            units::parse_service(
                parsed,
                &unit_path,
                units::UnitId(units::UnitIdKind::Service, next_id),
            )
            .map_err(|e| format!("{}", units::ParsingError::new(e, unit_path)))?
        } else if find_name.ends_with(".socket") {
            units::parse_socket(
                parsed,
                &unit_path,
                units::UnitId(units::UnitIdKind::Socket, next_id),
            )
            .map_err(|e| format!("{}", units::ParsingError::new(e, unit_path)))?
        } else if find_name.ends_with(".target") {
            units::parse_target(
                parsed,
                &unit_path,
                units::UnitId(units::UnitIdKind::Target, next_id),
            )
            .map_err(|e| format!("{}", units::ParsingError::new(e, unit_path)))?
        } else {
            return Err(format!(
                "File suffix not recognized for file {:?}",
                unit_path
            ));
        };

        Ok(unit)
    } else {
        Err(format!("Cannot find unit file for unit: {}", find_name))
    }
}

// check that all names referenced in the new units exist either in the old units
// or in the new units
fn check_all_names_exist(
    new_units: &HashMap<units::UnitId, units::Unit>,
    unit_table_locked: &units::UnitTable,
) -> Result<(), String> {
    let mut names_needed = Vec::new();
    for new_unit in new_units.values() {
        crate::units::collect_names_needed(new_unit, &mut names_needed);
    }
    let mut names_needed: std::collections::HashMap<_, _> =
        names_needed.iter().map(|name| (name, ())).collect();

    for unit in unit_table_locked.values() {
        let unit_locked = unit.lock().unwrap();
        for new_unit in new_units.values() {
            if unit_locked.id == new_unit.id {
                return Err(format!("Id {} exists already", new_unit.id));
            }
            if unit_locked.conf.name() == new_unit.conf.name() {
                return Err(format!("Name {} exists already", new_unit.conf.name()));
            }
        }
        if names_needed.contains_key(&unit_locked.conf.name()) {
            names_needed.remove(&unit_locked.conf.name()).unwrap();
        }
    }
    for unit in new_units.values() {
        if names_needed.contains_key(&unit.conf.name()) {
            names_needed.remove(&unit.conf.name()).unwrap();
        }
    }
    if names_needed.len() > 0 {
        return Err(format!(
            "Names referenced by unit but not found in the known set of units: {:?}",
            names_needed.keys().collect::<Vec<_>>()
        ));
    }
    Ok(())
}

/// Activates a new unit by
/// 1. (not yet but will be) checking the units referenced by this new unit
/// 1. inserting it into the unit_table of run_info
/// 1. activate the unit
/// 1. removing the unit again if the activation fails
pub fn insert_new_units(
    new_units: HashMap<units::UnitId, units::Unit>,
    run_info: units::ArcRuntimeInfo,
) -> Result<(), String> {
    // TODO check if new unit only refs existing units
    // TODO check if all ref'd units are not failed
    {
        let unit_table_locked = &mut *run_info.unit_table.write().unwrap();
        trace!("Check all names exist");
        check_all_names_exist(&new_units, &unit_table_locked)?;

        for (new_id, mut new_unit) in new_units.into_iter() {
            trace!("Add new unit: {}", new_unit.conf.name());
            // Setup relations of before <-> after / requires <-> requiredby
            for unit in unit_table_locked.values() {
                let mut unit_locked = unit.lock().unwrap();
                let name = unit_locked.conf.name();
                let id = unit_locked.id;
                if new_unit.conf.after.contains(&name) {
                    new_unit.install.after.push(id);
                    unit_locked.install.before.push(new_id);
                }
                if new_unit.conf.before.contains(&name) {
                    new_unit.install.before.push(id);
                    unit_locked.install.after.push(new_id);
                }
                if new_unit.conf.requires.contains(&name) {
                    new_unit.install.requires.push(id);
                    unit_locked.install.required_by.push(new_id);
                }
                if new_unit.conf.wants.contains(&name) {
                    new_unit.install.wants.push(id);
                    unit_locked.install.wanted_by.push(new_id);
                }
                if let Some(conf) = &new_unit.install.install_config {
                    if conf.required_by.contains(&name) {
                        new_unit.install.required_by.push(id);
                        unit_locked.install.requires.push(new_id);
                    }
                    if conf.wanted_by.contains(&name) {
                        new_unit.install.wanted_by.push(id);
                        unit_locked.install.wants.push(new_id);
                    }
                }
            }
            {
                unit_table_locked.insert(new_id, Arc::new(Mutex::new(new_unit)));
            }
            {
                let status_table_locked = &mut *run_info.status_table.write().unwrap();
                status_table_locked.insert(
                    new_id,
                    Arc::new(Mutex::new(units::UnitStatus::NeverStarted)),
                );
            }
        }
    }
    Ok(())
}
