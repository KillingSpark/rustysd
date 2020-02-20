use crate::units::*;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug)]
pub enum LoadingError {
    Parsing(ParsingError),
    Dependency(DependencyError),
}

#[derive(Debug)]
pub struct DependencyError {
    msg: String,
}

impl std::fmt::Display for DependencyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Dependency resolving error: {}", self.msg)
    }
}

impl std::convert::From<DependencyError> for LoadingError {
    fn from(s: DependencyError) -> Self {
        LoadingError::Dependency(s)
    }
}

impl std::convert::From<ParsingError> for LoadingError {
    fn from(s: ParsingError) -> Self {
        LoadingError::Parsing(s)
    }
}

pub fn load_all_units(
    paths: &[PathBuf],
    base_id: &mut u64,
    target_unit: &str,
) -> Result<HashMap<UnitId, Unit>, LoadingError> {
    let mut service_unit_table = HashMap::new();
    let mut socket_unit_table = HashMap::new();
    let mut target_unit_table = HashMap::new();
    for path in paths {
        parse_all_units(
            &mut service_unit_table,
            &mut socket_unit_table,
            &mut target_unit_table,
            path,
            base_id,
        )?;
    }

    let mut unit_table = std::collections::HashMap::new();
    unit_table.extend(service_unit_table);
    unit_table.extend(socket_unit_table);
    unit_table.extend(target_unit_table);
    fill_dependencies(&mut unit_table);

    prune_units(target_unit, &mut unit_table).unwrap();
    trace!("Finished pruning units");

    let mut service_unit_table = HashMap::new();
    let mut socket_unit_table = HashMap::new();
    let mut target_unit_table = HashMap::new();
    for (id, unit) in unit_table {
        match id.0 {
            UnitIdKind::Service => {
                service_unit_table.insert(id, unit);
            }
            UnitIdKind::Socket => {
                socket_unit_table.insert(id, unit);
            }
            UnitIdKind::Target => {
                target_unit_table.insert(id, unit);
            }
        }
    }

    apply_sockets_to_services(&mut service_unit_table, &mut socket_unit_table)
        .map_err(|e| DependencyError { msg: e })?;

    let removed_ids = prune_unused_sockets(&mut socket_unit_table);

    let mut unit_table = std::collections::HashMap::new();
    unit_table.extend(service_unit_table);
    unit_table.extend(socket_unit_table);
    unit_table.extend(target_unit_table);

    cleanup_removed_ids(&mut unit_table, &removed_ids);

    Ok(unit_table)
}

fn cleanup_removed_ids(
    units: &mut std::collections::HashMap<UnitId, Unit>,
    removed_ids: &Vec<UnitId>,
) {
    for unit in units.values_mut() {
        for id in removed_ids {
            while let Some(idx) = unit.install.after.iter().position(|el| *el == *id) {
                unit.install.after.remove(idx);
            }
            while let Some(idx) = unit.install.before.iter().position(|el| *el == *id) {
                unit.install.before.remove(idx);
            }
            while let Some(idx) = unit.install.wants.iter().position(|el| *el == *id) {
                unit.install.wants.remove(idx);
            }
            while let Some(idx) = unit.install.requires.iter().position(|el| *el == *id) {
                unit.install.requires.remove(idx);
            }
            while let Some(idx) = unit.install.wanted_by.iter().position(|el| *el == *id) {
                unit.install.wanted_by.remove(idx);
            }
            while let Some(idx) = unit.install.required_by.iter().position(|el| *el == *id) {
                unit.install.required_by.remove(idx);
            }
        }
    }
}

fn prune_unused_sockets(sockets: &mut std::collections::HashMap<UnitId, Unit>) -> Vec<UnitId> {
    let mut ids_to_remove = Vec::new();
    for unit in sockets.values() {
        if let UnitSpecialized::Socket(sock) = &unit.specialized {
            if sock.services.is_empty() {
                trace!(
                    "Prune socket {} because it was not added to any service",
                    unit.conf.name()
                );
                ids_to_remove.push(unit.id);
            }
        }
    }
    for id in &ids_to_remove {
        sockets.remove(id);
    }
    ids_to_remove
}

fn parse_all_units(
    services: &mut std::collections::HashMap<UnitId, Unit>,
    sockets: &mut std::collections::HashMap<UnitId, Unit>,
    targets: &mut std::collections::HashMap<UnitId, Unit>,
    path: &PathBuf,
    last_id: &mut u64,
) -> Result<(), ParsingError> {
    let files = get_file_list(path)
        .map_err(|e| ParsingError::new(ParsingErrorReason::from(e), path.clone()))?;
    for entry in files {
        if entry.path().is_dir() {
            parse_all_units(services, sockets, targets, path, last_id)?;
        } else {
            let raw = std::fs::read_to_string(&entry.path()).map_err(|e| {
                ParsingError::new(ParsingErrorReason::from(Box::new(e)), path.clone())
            })?;

            let parsed_file = parse_file(&raw)
                .map_err(|e| ParsingError::new(ParsingErrorReason::from(e), path.clone()))?;

            if entry.path().to_str().unwrap().ends_with(".service") {
                *last_id += 1;
                trace!("ID {}: {:?}", last_id, entry.path());
                let new_id = UnitId(UnitIdKind::Service, *last_id);
                services.insert(
                    new_id,
                    parse_service(parsed_file, &entry.path(), new_id.clone()).map_err(|e| {
                        ParsingError::new(ParsingErrorReason::from(e), path.clone())
                    })?,
                );
            } else if entry.path().to_str().unwrap().ends_with(".socket") {
                *last_id += 1;
                trace!("ID {}: {:?}", last_id, entry.path());
                let new_id = UnitId(UnitIdKind::Socket, *last_id);
                sockets.insert(
                    new_id,
                    parse_socket(parsed_file, &entry.path(), new_id.clone()).map_err(|e| {
                        ParsingError::new(ParsingErrorReason::from(e), path.clone())
                    })?,
                );
            } else if entry.path().to_str().unwrap().ends_with(".target") {
                *last_id += 1;
                trace!("ID {}: {:?}", last_id, entry.path());
                let new_id = UnitId(UnitIdKind::Target, *last_id);
                targets.insert(
                    new_id,
                    parse_target(parsed_file, &entry.path(), new_id.clone()).map_err(|e| {
                        ParsingError::new(ParsingErrorReason::from(e), path.clone())
                    })?,
                );
            }
        }
    }
    Ok(())
}
