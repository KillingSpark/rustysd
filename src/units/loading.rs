use crate::units::*;
use std::collections::HashMap;
use std::path::PathBuf;

pub fn load_all_units(paths: &[PathBuf]) -> Result<HashMap<UnitId, Unit>, ParsingError> {
    let mut base_id = 0;
    let mut service_unit_table = HashMap::new();
    let mut socket_unit_table = HashMap::new();
    let mut target_unit_table = HashMap::new();
    for path in paths {
        parse_all_units(
            &mut service_unit_table,
            &mut socket_unit_table,
            &mut target_unit_table,
            path,
            &mut base_id,
        )?;
    }

    let mut socket_target_unit = None;
    for target in target_unit_table.values_mut() {
        if target.conf.name() == "sockets.target" {
            socket_target_unit = Some(target);
            break;
        }
    }

    if let Some(socket_target_unit) = socket_target_unit {
        trace!("Adding sockets.target");
        for sock in socket_unit_table.values_mut() {
            sock.install.before.push(socket_target_unit.id);
            sock.install.required_by.push(socket_target_unit.id);
            socket_target_unit.install.after.push(sock.id);
            socket_target_unit.install.requires.push(sock.id);
        }
    }

    apply_sockets_to_services(&mut service_unit_table, &mut socket_unit_table)?;
    let mut unit_table = std::collections::HashMap::new();
    unit_table.extend(service_unit_table);
    unit_table.extend(socket_unit_table);
    unit_table.extend(target_unit_table);
    fill_dependencies(&mut unit_table);
    Ok(unit_table)
}

fn parse_all_units(
    services: &mut std::collections::HashMap<UnitId, Unit>,
    sockets: &mut std::collections::HashMap<UnitId, Unit>,
    targets: &mut std::collections::HashMap<UnitId, Unit>,
    path: &PathBuf,
    last_id: &mut u64,
) -> Result<(), ParsingError> {
    let files = get_file_list(path)?;
    for entry in files {
        if entry.path().is_dir() {
            parse_all_units(services, sockets, targets, path, last_id)?;
        } else {
            let raw = std::fs::read_to_string(&entry.path()).map_err(|e| ParsingError {
                msg: Some(format!("Error opening file: {:?} error: {}", path, e)),
                reason: Some(Box::new(e)),
            })?;

            let parsed_file = parse_file(&raw).map_err(|err| ParsingError {
                reason: Some(Box::new(err)),
                msg: Some(format!("In file: {:?}", path)),
            })?;

            if entry.path().to_str().unwrap().ends_with(".service") {
                *last_id += 1;
                trace!("{:?}, {}", entry.path(), last_id);
                let new_id = UnitId(UnitIdKind::Service, *last_id);
                services.insert(
                    new_id,
                    parse_service(parsed_file, &entry.path(), new_id.clone())?,
                );
            } else if entry.path().to_str().unwrap().ends_with(".socket") {
                *last_id += 1;
                trace!("{:?}, {}", entry.path(), last_id);
                let new_id = UnitId(UnitIdKind::Service, *last_id);
                sockets.insert(
                    new_id,
                    parse_socket(parsed_file, &entry.path(), new_id.clone())?,
                );
            } else if entry.path().to_str().unwrap().ends_with(".target") {
                *last_id += 1;
                trace!("{:?}, {}", entry.path(), last_id);
                let new_id = UnitId(UnitIdKind::Service, *last_id);
                targets.insert(
                    new_id,
                    parse_target(parsed_file, &entry.path(), new_id.clone())?,
                );
            }
        }
    }
    Ok(())
}
