use crate::units::*;
use std::collections::HashMap;

type SocketTable = HashMap<UnitId, Unit>;
type ServiceTable = HashMap<UnitId, Unit>;

#[allow(dead_code)]
pub fn prune_units(
    target_unit_name: &str,
    unit_table: &mut HashMap<UnitId, Unit>,
) -> Result<(), String> {
    let mut ids_to_keep = Vec::new();
    let startunit = unit_table.values().fold(None, |mut result, unit| {
        if unit.conf.name() == target_unit_name {
            result = Some(unit.id);
        }
        result
    });
    let startunit_id = if let Some(startunit) = startunit {
        startunit
    } else {
        return Err(format!("Target unit {} not found", target_unit_name));
    };

    find_needed_units_recursive(startunit_id, unit_table, &mut ids_to_keep);

    let mut ids_to_remove = Vec::new();
    for id in unit_table.keys() {
        if !ids_to_keep.contains(id) {
            ids_to_remove.push(*id);
        }
    }
    for id in &ids_to_remove {
        let unit = unit_table.remove(id).unwrap();
        trace!("Pruning unit: {}", unit.conf.name());
    }

    add_implicit_before_after(unit_table);
    for unit in unit_table.values_mut() {
        unit.install.before = unit
            .install
            .before
            .iter()
            .filter(|id| ids_to_keep.contains(id))
            .map(|id| *id)
            .collect();

        unit.install.after = unit
            .install
            .after
            .iter()
            .filter(|id| ids_to_keep.contains(id))
            .map(|id| *id)
            .collect();

        unit.install.requires = unit
            .install
            .requires
            .iter()
            .filter(|id| ids_to_keep.contains(id))
            .map(|id| *id)
            .collect();

        unit.install.wants = unit
            .install
            .wants
            .iter()
            .filter(|id| ids_to_keep.contains(id))
            .map(|id| *id)
            .collect();

        unit.install.required_by = unit
            .install
            .required_by
            .iter()
            .filter(|id| ids_to_keep.contains(id))
            .map(|id| *id)
            .collect();

        unit.install.wanted_by = unit
            .install
            .wanted_by
            .iter()
            .filter(|id| ids_to_keep.contains(id))
            .map(|id| *id)
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
    visited_ids.push(needed_id);

    let unit = unit_table.get(&needed_id).unwrap();
    let mut new_needed_ids = Vec::new();

    for new_id in &unit.install.requires {
        new_needed_ids.push(*new_id);
    }
    for new_id in &unit.install.wants {
        new_needed_ids.push(*new_id);
    }
    for new_id in &unit.install.required_by {
        new_needed_ids.push(*new_id);
    }
    for new_id in &unit.install.wanted_by {
        new_needed_ids.push(*new_id);
    }
    new_needed_ids.sort();
    new_needed_ids.dedup();

    trace!("Id {:?} references ids: {:?}", needed_id, new_needed_ids);

    for new_id in &new_needed_ids {
        find_needed_units_recursive(*new_id, unit_table, visited_ids);
    }
}

// add after/before relations for required_by/wanted_by relations after pruning
pub fn add_implicit_before_after(units: &mut HashMap<UnitId, Unit>) {
    let mut name_to_id = HashMap::new();

    for (id, unit) in &*units {
        let name = unit.conf.name();
        name_to_id.insert(name, *id);
    }

    let mut before = Vec::new();
    let mut after = Vec::new();
    for unit in (*units).values_mut() {
        if let Some(conf) = &unit.install.install_config {
            for name in &conf.wanted_by {
                let id = name_to_id[name.as_str()];
                before.push((id, unit.id));
                after.push((unit.id, id));
            }
            for name in &conf.required_by {
                let id = name_to_id[name.as_str()];
                before.push((id, unit.id));
                after.push((unit.id, id));
            }
        }
    }

    for (before, after) in before {
        let unit = units.get_mut(&after).unwrap();
        unit.install.before.push(before);
    }
    for (after, before) in after {
        let unit = units.get_mut(&before).unwrap();
        unit.install.after.push(after);
    }
}

// make edges between units visible on bot sides: required <-> required_by  after <-> before
pub fn fill_dependencies(units: &mut HashMap<UnitId, Unit>) {
    let mut name_to_id = HashMap::new();

    for (id, unit) in &*units {
        let name = unit.conf.name();
        name_to_id.insert(name, *id);
    }

    let mut required_by = Vec::new();
    let mut wanted_by: Vec<(UnitId, UnitId)> = Vec::new();
    let mut before = Vec::new();
    let mut after = Vec::new();

    for unit in (*units).values_mut() {
        let conf = &unit.conf;
        for name in &conf.wants {
            let id = name_to_id[name.as_str()];
            unit.install.wants.push(id);
            wanted_by.push((id, unit.id));
        }
        for name in &conf.requires {
            let id = name_to_id[name.as_str()];
            unit.install.requires.push(id);
            required_by.push((id, unit.id));
        }
        for name in &conf.before {
            let id = name_to_id[name.as_str()];
            unit.install.before.push(id);
            after.push((unit.id, id))
        }
        for name in &conf.after {
            let id = name_to_id[name.as_str()];
            unit.install.after.push(id);
            before.push((unit.id, id))
        }

        if let Some(conf) = &unit.install.install_config {
            for name in &conf.wanted_by {
                let id = name_to_id[name.as_str()];
                wanted_by.push((unit.id, id));
            }
            for name in &conf.required_by {
                let id = name_to_id[name.as_str()];
                required_by.push((unit.id, id));
            }
        }
    }

    for (wanted, wanting) in wanted_by {
        let unit = units.get_mut(&wanting).unwrap();
        unit.install.wants.push(wanted);
        let unit = units.get_mut(&wanted).unwrap();
        unit.install.wanted_by.push(wanting);
    }

    for (required, requiring) in required_by {
        let unit = units.get_mut(&requiring).unwrap();
        unit.install.requires.push(required);
        let unit = units.get_mut(&required).unwrap();
        unit.install.required_by.push(requiring);
    }

    for (before, after) in before {
        let unit = units.get_mut(&after).unwrap();
        unit.install.before.push(before);
    }
    for (after, before) in after {
        let unit = units.get_mut(&before).unwrap();
        unit.install.after.push(after);
    }

    for srvc in units.values_mut() {
        srvc.dedup_dependencies();
    }
}

fn add_sock_srvc_relations(
    srvc_id: UnitId,
    srvc_install: &mut Install,
    sock_id: UnitId,
    sock_install: &mut Install,
) {
    srvc_install.after.push(sock_id);
    srvc_install.requires.push(sock_id);
    sock_install.before.push(srvc_id);
    sock_install.required_by.push(srvc_id);
}

pub fn apply_sockets_to_services(
    service_table: &mut ServiceTable,
    socket_table: &mut SocketTable,
) -> Result<(), String> {
    for sock_unit in socket_table.values_mut() {
        let mut counter = 0;

        if let UnitSpecialized::Socket(sock) = &mut sock_unit.specialized {
            trace!("Searching services for socket: {}", sock_unit.conf.name());
            for srvc_unit in service_table.values_mut() {
                let srvc = &mut srvc_unit.specialized;
                if let UnitSpecialized::Service(srvc) = srvc {
                    // add sockets for services with the exact same name
                    if (srvc_unit.conf.name_without_suffix()
                        == sock_unit.conf.name_without_suffix())
                        && !srvc.socket_names.contains(&sock_unit.conf.name())
                    {
                        trace!(
                            "add socket: {} to service: {}",
                            sock_unit.conf.name(),
                            srvc_unit.conf.name()
                        );

                        srvc.socket_names.push(sock_unit.conf.name());
                        sock.services.push(srvc_unit.conf.name());
                        add_sock_srvc_relations(
                            srvc_unit.id,
                            &mut srvc_unit.install,
                            sock_unit.id,
                            &mut sock_unit.install,
                        );
                        counter += 1;
                    }

                    // add sockets to services that specify that the socket belongs to them
                    if srvc.service_config.sockets.contains(&sock_unit.conf.name())
                        && !srvc.socket_names.contains(&sock_unit.conf.name())
                    {
                        trace!(
                            "add socket: {} to service: {}",
                            sock_unit.conf.name(),
                            srvc_unit.conf.name()
                        );
                        srvc.socket_names.push(sock_unit.conf.name());
                        sock.services.push(srvc_unit.conf.name());
                        add_sock_srvc_relations(
                            srvc_unit.id,
                            &mut srvc_unit.install,
                            sock_unit.id,
                            &mut sock_unit.install,
                        );
                        counter += 1;
                    }
                }
            }

            // add socket to the specified services
            for srvc_name in &sock.services {
                for srvc_unit in service_table.values_mut() {
                    let srvc = &mut srvc_unit.specialized;
                    if let UnitSpecialized::Service(srvc) = srvc {
                        if (*srvc_name == srvc_unit.conf.name())
                            && !srvc.socket_names.contains(&sock_unit.conf.name())
                        {
                            trace!(
                                "add socket: {} to service: {}",
                                sock_unit.conf.name(),
                                srvc_unit.conf.name()
                            );

                            srvc.socket_names.push(sock_unit.conf.name());
                            add_sock_srvc_relations(
                                srvc_unit.id,
                                &mut srvc_unit.install,
                                sock_unit.id,
                                &mut sock_unit.install,
                            );
                            counter += 1;
                        }
                    }
                }
            }
        }
        if counter > 1 {
            return Err(format!(
                "Added socket: {} to too many services (should be at most one): {}",
                sock_unit.conf.name(),
                counter
            ));
        }
        if counter == 0 {
            warn!("Added socket: {} to no service", sock_unit.conf.name());
        }
    }

    for srvc_unit in service_table.values_mut() {
        if let UnitSpecialized::Service(srvc) = &mut srvc_unit.specialized {
            srvc.socket_names.sort();
        }
    }

    Ok(())
}
