use crate::units;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

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
            let new_id = units::UnitId {
                kind: units::UnitIdKind::Service,
                name: unit_path.file_name().unwrap().to_str().unwrap().to_owned(),
            };
            units::parse_service(parsed, &unit_path, new_id)
                .map_err(|e| format!("{}", units::ParsingError::new(e, unit_path)))?
                .into()
        } else if find_name.ends_with(".socket") {
            let new_id = units::UnitId {
                kind: units::UnitIdKind::Socket,
                name: unit_path.file_name().unwrap().to_str().unwrap().to_owned(),
            };
            units::parse_socket(parsed, &unit_path, new_id)
                .map_err(|e| format!("{}", units::ParsingError::new(e, unit_path)))?
                .into()
        } else if find_name.ends_with(".target") {
            let new_id = units::UnitId {
                kind: units::UnitIdKind::Target,
                name: unit_path.file_name().unwrap().to_str().unwrap().to_owned(),
            };
            units::parse_target(parsed, &unit_path, new_id)
                .map_err(|e| format!("{}", units::ParsingError::new(e, unit_path)))?
                .into()
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
        for new_unit in new_units.values() {
            if unit.id == new_unit.id {
                return Err(format!("Id {} exists already", new_unit.id));
            }
            if unit.id.name == new_unit.id.name {
                return Err(format!("Name {} exists already", new_unit.id.name));
            }
        }
        if names_needed.contains_key(&unit.id.name) {
            names_needed.remove(&unit.id.name).unwrap();
        }
    }
    for unit in new_units.values() {
        if names_needed.contains_key(&unit.id.name) {
            names_needed.remove(&unit.id.name).unwrap();
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
    run_info: &mut units::RuntimeInfo,
) -> Result<(), String> {
    // TODO check if new unit only refs existing units
    // TODO check if all ref'd units are not failed
    {
        let unit_table = &mut run_info.unit_table;
        trace!("Check all names exist");
        check_all_names_exist(&new_units, unit_table)?;

        for (new_id, mut new_unit) in new_units.into_iter() {
            trace!("Add new unit: {}", new_unit.id.name);
            // Setup relations of before <-> after / requires <-> requiredby
            for unit in unit_table.values() {
                if new_unit.common.dependencies.after.contains(&unit.id) {
                    unit.common.dependencies.before.push(new_id);
                }
                if new_unit.common.dependencies.before.contains(&unit.id) {
                    unit.common.dependencies.after.push(new_id);
                }
                if new_unit.common.dependencies.requires.contains(&unit.id) {
                    unit.common.dependencies.required_by.push(new_id);
                }
                if new_unit.common.dependencies.wants.contains(&unit.id) {
                    unit.common.dependencies.wanted_by.push(new_id);
                }
                if new_unit.common.dependencies.required_by.contains(&unit.id) {
                    unit.common.dependencies.requires.push(new_id);
                }
                if new_unit.common.dependencies.wanted_by.contains(&unit.id) {
                    unit.common.dependencies.wants.push(new_id);
                }
            }
            unit_table.insert(new_id, new_unit);
        }
    }
    Ok(())
}
