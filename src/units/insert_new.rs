use crate::units;
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
        let content = fs::read_to_string(&unit_path)
            .map_err(|e| format!("Error while reading file {:?}: {}", unit_path, e))?;
        let parsed = units::parse_file(&content)
            .map_err(|e| format!("Error while parsing unit file {:?}: {}", unit_path, e))?;
        let unit = if find_name.ends_with(".service") {
            units::parse_service(
                parsed,
                &unit_path,
                units::UnitId(units::UnitIdKind::Service, next_id),
            )
            .map_err(|e| format!("Error while parsing unit file {:?}: {}", unit_path, e))?
        } else if find_name.ends_with(".socket") {
            units::parse_service(
                parsed,
                &unit_path,
                units::UnitId(units::UnitIdKind::Service, next_id),
            )
            .map_err(|e| format!("Error while parsing unit file {:?}: {}", unit_path, e))?
        } else if find_name.ends_with(".target") {
            
                units::parse_service(
                    parsed,
                    &unit_path,
                    units::UnitId(units::UnitIdKind::Service, next_id),
                )
                .map_err(|e| format!("Error while parsing unit file {:?}: {}", unit_path, e))?
        } else {
            return Err(format!("File suffix not recognized for file {:?}", unit_path));
        };

        Ok(unit)
    } else {
        Err(format!("Cannot find unit file for unit: {}", find_name))
    }
}

/// Activates a new unit by 
/// 1. (not yet but will be) checking the units referenced by this new unit 
/// 1. inserting it into the unit_table of run_info
/// 1. activate the unit
/// 1. removing the unit again if the activation fails
pub fn activate_new_unit(new_unit: units::Unit, run_info: units::ArcRuntimeInfo) -> Result<(), String> {
    let new_id = new_unit.id; 
    // TODO check if new unit only refs existing units
    // TODO check if all ref'd units are not failed
    {
        let unit_table_locked = &mut *run_info.unit_table.write().unwrap();
        unit_table_locked.insert(new_id, Arc::new(Mutex::new(new_unit)));
    }
    Ok(())
}
