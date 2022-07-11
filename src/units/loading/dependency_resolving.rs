use log::trace;
use log::warn;

use crate::runtime_info::*;
use crate::units::*;

use std::collections::HashMap;
use std::convert::TryInto;

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
    let mut ids_to_keep = vec![startunit_id.clone()];
    crate::units::collect_unit_start_subgraph(&mut ids_to_keep, unit_table);

    // walk the tree along the wants/requires/before/... relations and record which ids are needed
    //find_needed_units_recursive(startunit_id, unit_table, &mut ids_to_keep);

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

    // Cleanup all removed IDs
    for unit in unit_table.values_mut() {
        match &mut unit.specific {
            Specific::Service(specific) => {
                specific.conf.sockets = specific
                    .conf
                    .sockets
                    .iter()
                    .filter(|id| ids_to_keep.contains(id))
                    .cloned()
                    .collect()
            }
            Specific::Socket(specific) => {
                specific.conf.services = specific
                    .conf
                    .services
                    .iter()
                    .filter(|id| ids_to_keep.contains(id))
                    .cloned()
                    .collect()
            }
            Specific::Target(_) => { /**/ }
        }

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

/// make edges between units visible on bot sides: required <-> required_by  after <-> before
///
/// Also adds all implicit dependencies between units (currently only a subset of the ones defined
/// by systemd)
pub fn fill_dependencies(units: &mut HashMap<UnitId, Unit>) -> Result<(), String> {
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

    add_all_implicit_relations(units)?;

    for srvc in units.values_mut() {
        srvc.dedup_dependencies();
    }

    Ok(())
}

/// Function to apply all implicit relations to the units in the table
///
/// This is currently only a subset of all implicit relations systemd applies
fn add_all_implicit_relations(units: &mut UnitTable) -> Result<(), String> {
    add_socket_target_relations(units);
    apply_sockets_to_services(units)?;
    Ok(())
}

/// There is an implicit *.socket before sockets.target relation
///
/// This is only applied if this target exists. I would like to
/// leave well known units as optional as possible but this is needed
/// for compatibility
fn add_socket_target_relations(units: &mut UnitTable) {
    let target_id: UnitId = "sockets.target".try_into().unwrap();
    let mut socket_ids = Vec::new();
    if units.contains_key(&target_id) {
        for unit in units.values_mut() {
            if UnitIdKind::Socket == unit.id.kind {
                // Add to socket
                unit.common.dependencies.before.push(target_id.clone());
                unit.common.dependencies.dedup();
                // Remember socket id to add to the target
                socket_ids.push(unit.id.clone());
            }
        }
        let target = units.get_mut(&target_id).unwrap();
        target.common.dependencies.after.extend(socket_ids);
        target.common.dependencies.dedup();
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

    sock_install.dedup();
    srvc_install.dedup();

    if !srvc_conf.sockets.contains(&sock_id) {
        srvc_conf.sockets.push(sock_id);
    }
    if !sock_conf.services.contains(&srvc_id) {
        sock_conf.services.push(srvc_id);
    }
}

/// This takes a set of services and sockets and matches them both by their name and their
/// respective explicit settings. It adds appropriate before/after and requires/required_by relations.
fn apply_sockets_to_services(unit_table: &mut UnitTable) -> Result<(), String> {
    let mut service_ids = Vec::new();
    let mut socket_ids = Vec::new();
    for id in unit_table.keys() {
        match id.kind {
            UnitIdKind::Service => {
                service_ids.push(id.clone());
            }
            UnitIdKind::Socket => {
                socket_ids.push(id.clone());
            }
            UnitIdKind::Target => {
                // ignore targets here
            }
        }
    }

    for sock_unit in &socket_ids {
        let mut sock_unit = unit_table.remove(sock_unit).unwrap();
        let mut counter = 0;

        if let Specific::Socket(sock) = &mut sock_unit.specific {
            trace!("Searching services for socket: {}", sock_unit.id.name);
            for srvc_unit in &service_ids {
                let mut srvc_unit = unit_table.remove(srvc_unit).unwrap();

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
                    if (srvc.conf.sockets.contains(&sock_unit.id)
                        && !sock.conf.services.contains(&srvc_unit.id))
                        || (sock.conf.services.contains(&srvc_unit.id)
                            && !srvc.conf.sockets.contains(&sock_unit.id))
                    {
                        trace!(
                            "add socket: {} to service: {} because one mentions the other",
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
                }
                unit_table.insert(srvc_unit.id.clone(), srvc_unit);
            }
        }
        let sock_name = sock_unit.id.name.clone();
        unit_table.insert(sock_unit.id.clone(), sock_unit);
        if counter > 1 {
            return Err(format!(
                "Added socket: {} to too many services (should be at most one): {}",
                sock_name, counter
            ));
        }
        if counter == 0 {
            warn!("Added socket: {} to no service", sock_name);
        }
    }

    Ok(())
}
