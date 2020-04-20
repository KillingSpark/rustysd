use crate::units::*;
use std::collections::HashMap;

type SocketTable = HashMap<UnitId, Unit>;
type ServiceTable = HashMap<UnitId, Unit>;

/// Takes a set of units and prunes those that are not needed to reach the specified target unit.
pub fn prune_units(
    target_unit_name: &str,
    unit_table: &mut HashMap<UnitId, Unit>,
) -> Result<(), String> {
    let startunit = unit_table.values().fold(None, |mut result, unit| {
        if unit.id.name == target_unit_name {
            result = Some(unit.id.clone());
        }
        result
    });
    let startunit_id = if let Some(startunit) = startunit {
        startunit
    } else {
        return Err(format!("Target unit {} not found", target_unit_name));
    };
    // This vec will record the unit ids that will be kept
    let mut ids_to_keep = Vec::new();

    // walk the tree along the wants/requires/before/... relations and record which ids are needed
    find_needed_units_recursive(startunit_id, unit_table, &mut ids_to_keep);

    // Remove all units that have been deemed unnecessary
    let mut ids_to_remove = Vec::new();
    for id in unit_table.keys() {
        if !ids_to_keep.contains(id) {
            ids_to_remove.push(id.clone());
        }
    }
    for id in &ids_to_remove {
        let unit = unit_table.remove(id).unwrap();
        trace!("Pruning unit: {}", unit.id.name);
    }

    add_implicit_before_after(unit_table);

    // Cleanup all removed IDs
    for unit in unit_table.values_mut() {
        unit.common.dependencies.before = unit
            .common
            .dependencies
            .before
            .iter()
            .filter(|id| ids_to_keep.contains(id))
            .map(|id| id.clone())
            .collect();

        unit.common.dependencies.after = unit
            .common
            .dependencies
            .after
            .iter()
            .filter(|id| ids_to_keep.contains(id))
            .map(|id| id.clone())
            .collect();

        unit.common.dependencies.requires = unit
            .common
            .dependencies
            .requires
            .iter()
            .filter(|id| ids_to_keep.contains(id))
            .map(|id| id.clone())
            .collect();

        unit.common.dependencies.wants = unit
            .common
            .dependencies
            .wants
            .iter()
            .filter(|id| ids_to_keep.contains(id))
            .map(|id| id.clone())
            .collect();

        unit.common.dependencies.required_by = unit
            .common
            .dependencies
            .required_by
            .iter()
            .filter(|id| ids_to_keep.contains(id))
            .map(|id| id.clone())
            .collect();

        unit.common.dependencies.wanted_by = unit
            .common
            .dependencies
            .wanted_by
            .iter()
            .filter(|id| ids_to_keep.contains(id))
            .map(|id| id.clone())
            .collect();

        unit.dedup_dependencies();
    }
    Ok(())
}

fn find_needed_units_recursive(
    needed_id: UnitId,
    unit_table: &HashMap<UnitId, Unit>,
    visited_ids: &mut Vec<UnitId>,
) {
    if visited_ids.contains(&needed_id) {
        return;
    }
    visited_ids.push(needed_id.clone());

    let unit = unit_table.get(&needed_id).unwrap();
    let mut new_needed_ids = Vec::new();

    for new_id in &unit.common.dependencies.requires {
        new_needed_ids.push(new_id.clone());
    }
    for new_id in &unit.common.dependencies.wants {
        new_needed_ids.push(new_id.clone());
    }
    for new_id in &unit.common.dependencies.required_by {
        new_needed_ids.push(new_id.clone());
    }
    for new_id in &unit.common.dependencies.wanted_by {
        new_needed_ids.push(new_id.clone());
    }
    new_needed_ids.sort();
    new_needed_ids.dedup();

    trace!("Id {:?} references ids: {:?}", needed_id, new_needed_ids);

    for new_id in &new_needed_ids {
        find_needed_units_recursive(new_id.clone(), unit_table, visited_ids);
    }
}

// add after/before relations for required_by/wanted_by relations after pruning
pub fn add_implicit_before_after(units: &mut HashMap<UnitId, Unit>) {
    let mut before = Vec::new();
    let mut after = Vec::new();
    for unit in (*units).values_mut() {
        for id in &unit.common.dependencies.wanted_by {
            before.push((id.clone(), unit.id.clone()));
            after.push((unit.id.clone(), id.clone()));
        }
        for id in &unit.common.dependencies.required_by {
            before.push((id.clone(), unit.id.clone()));
            after.push((unit.id.clone(), id.clone()));
        }
    }

    for (before, after) in before {
        let unit = units.get_mut(&after).unwrap();
        unit.common.dependencies.before.push(before);
    }
    for (after, before) in after {
        let unit = units.get_mut(&before).unwrap();
        unit.common.dependencies.after.push(after);
    }
}

// make edges between units visible on bot sides: required <-> required_by  after <-> before
pub fn fill_dependencies(units: &mut HashMap<UnitId, Unit>) {
    let mut required_by = Vec::new();
    let mut wanted_by: Vec<(UnitId, UnitId)> = Vec::new();
    let mut before = Vec::new();
    let mut after = Vec::new();

    for unit in (*units).values_mut() {
        trace!("Fill deps for unit: {:?}", unit.id);
        let conf = &mut unit.common.dependencies;
        for id in &conf.wants {
            wanted_by.push((id.clone(), unit.id.clone()));
        }
        for id in &conf.requires {
            required_by.push((id.clone(), unit.id.clone()));
        }
        for id in &conf.before {
            after.push((unit.id.clone(), id.clone()))
        }
        for id in &conf.after {
            before.push((unit.id.clone(), id.clone()))
        }
        for id in &conf.wanted_by {
            wanted_by.push((unit.id.clone(), id.clone()));
        }
        for id in &conf.required_by {
            required_by.push((unit.id.clone(), id.clone()));
        }
    }

    for (wanted, wanting) in wanted_by {
        trace!("{:?} wants {:?}", wanting, wanted);
        let unit = units.get_mut(&wanting).unwrap();
        unit.common.dependencies.wants.push(wanted.clone());
        let unit = units.get_mut(&wanted).unwrap();
        unit.common.dependencies.wanted_by.push(wanting);
    }

    for (required, requiring) in required_by {
        let unit = units.get_mut(&requiring).unwrap();
        unit.common.dependencies.requires.push(required.clone());
        let unit = units.get_mut(&required).unwrap();
        unit.common.dependencies.required_by.push(requiring);
    }

    for (before, after) in before {
        let unit = units.get_mut(&after).unwrap();
        unit.common.dependencies.before.push(before);
    }
    for (after, before) in after {
        let unit = units.get_mut(&before).unwrap();
        unit.common.dependencies.after.push(after);
    }

    for srvc in units.values_mut() {
        srvc.dedup_dependencies();
    }
}

fn add_sock_srvc_relations(
    srvc_id: UnitId,
    srvc_install: &mut Dependencies,
    srvc_conf: &mut ServiceConfig,
    sock_id: UnitId,
    sock_install: &mut Dependencies,
    sock_conf: &mut SocketConfig,
) {
    srvc_install.after.push(sock_id.clone());
    srvc_install.requires.push(sock_id.clone());
    sock_install.before.push(srvc_id.clone());
    sock_install.required_by.push(srvc_id.clone());

    srvc_conf.sockets.push(srvc_id.name.clone());
    sock_conf.services.push(sock_id.name.clone());
}

/// This takes a set of services and sockets and matches them both by their name and their
/// respective explicit settings. It adds appropriate before/after and requires/required_by relations.
pub fn apply_sockets_to_services(
    service_table: &mut ServiceTable,
    socket_table: &mut SocketTable,
) -> Result<(), String> {
    for sock_unit in socket_table.values_mut() {
        let mut counter = 0;

        if let Specific::Socket(sock) = &mut sock_unit.specific {
            trace!("Searching services for socket: {}", sock_unit.id.name);
            for srvc_unit in service_table.values_mut() {
                let srvc = &mut srvc_unit.specific;
                if let Specific::Service(srvc) = srvc {
                    // add sockets for services with the exact same name
                    if (srvc_unit.id.name_without_suffix() == sock_unit.id.name_without_suffix())
                        && !srvc.has_socket(&sock_unit.id.name)
                    {
                        trace!(
                            "add socket: {} to service: {} because their names match",
                            sock_unit.id.name,
                            srvc_unit.id.name
                        );

                        add_sock_srvc_relations(
                            srvc_unit.id.clone(),
                            &mut srvc_unit.common.dependencies,
                            &mut srvc.conf,
                            sock_unit.id.clone(),
                            &mut sock_unit.common.dependencies,
                            &mut sock.conf,
                        );
                        counter += 1;
                    }

                    // add sockets to services that specify that the socket belongs to them
                    // or sockets to services that specify that they belong to the service
                    if (srvc.conf.sockets.contains(&sock_unit.id.name)
                        && !sock.conf.services.contains(&srvc_unit.id.name))
                        || (sock.conf.services.contains(&srvc_unit.id.name)
                            && !srvc.conf.sockets.contains(&sock_unit.id.name))
                    {
                        trace!(
                            "add socket: {} to service: {} because one mentions the other",
                            sock_unit.id.name,
                            srvc_unit.id.name
                        );
                        sock.conf.services.push(srvc_unit.id.name.clone());
                        add_sock_srvc_relations(
                            srvc_unit.id.clone(),
                            &mut srvc_unit.common.dependencies,
                            &mut srvc.conf,
                            sock_unit.id.clone(),
                            &mut sock_unit.common.dependencies,
                            &mut sock.conf,
                        );
                        counter += 1;
                    }
                }
            }
        }
        if counter > 1 {
            return Err(format!(
                "Added socket: {} to too many services (should be at most one): {}",
                sock_unit.id.name, counter
            ));
        }
        if counter == 0 {
            warn!("Added socket: {} to no service", sock_unit.id.name);
        }
    }

    for srvc_unit in service_table.values_mut() {
        if let Specific::Service(srvc) = &mut srvc_unit.specific {
            srvc.conf.sockets.sort();
        }
    }

    Ok(())
}
