//! Activate units (recursively and parallel along the dependency tree)

use super::units::*;
use crate::platform::EventFd;
use std::collections::HashMap;
use std::sync::Arc;
use threadpool::ThreadPool;

fn activate_units_recursive(
    ids_to_start: Vec<UnitId>,
    run_info: ArcRuntimeInfo,
    tpool: ThreadPool,
    notification_socket_path: std::path::PathBuf,
    eventfds: Arc<Vec<EventFd>>,
) {
    for id in ids_to_start {
        let run_info_copy = run_info.clone();
        let tpool_copy = tpool.clone();
        let note_sock_copy = notification_socket_path.clone();
        let eventfds_copy = eventfds.clone();
        tpool.execute(move || {
            let run_info_copy2 = run_info_copy.clone();
            let tpool_copy2 = tpool_copy.clone();
            let note_sock_copy2 = note_sock_copy.clone();
            let eventfds_copy2 = eventfds_copy.clone();

            match activate_unit(id, run_info_copy, note_sock_copy, eventfds_copy, true) {
                Ok(StartResult::Started(next_services_ids)) => {
                    let next_services_job = move || {
                        activate_units_recursive(
                            next_services_ids,
                            run_info_copy2,
                            tpool_copy2,
                            note_sock_copy2,
                            eventfds_copy2,
                        );
                    };
                    tpool_copy.execute(next_services_job);
                }
                Ok(StartResult::Ignored) => {
                    // Thats ok
                }
                Err(e) => {
                    panic!("Error while activating unit {}", e);
                }
            }
        });
    }
}

pub enum StartResult {
    Started(Vec<UnitId>),
    Ignored,
}

pub fn activate_unit(
    id_to_start: UnitId,
    run_info: ArcRuntimeInfo,
    notification_socket_path: std::path::PathBuf,
    eventfds: Arc<Vec<EventFd>>,
    allow_ignore: bool,
) -> std::result::Result<StartResult, std::string::String> {
    trace!("Activate id: {:?}", id_to_start);

    // 1) First lock the unit itself
    // 1.5) Check if this unit should be started right now
    // 2) Then lock the needed other units (only for sockets of services right now)
    // With that we always maintain a consistent order between locks so deadlocks shouldnt occur
    let unit = {
        let units_locked = run_info.unit_table.read().unwrap();
        match units_locked.get(&id_to_start) {
            Some(unit) => Arc::clone(unit),
            None => {
                panic!("Tried to run a unit that has been removed from the map");
            }
        }
    };
    trace!("Lock unit: {}", id_to_start);
    let mut unit_locked = unit.lock().unwrap();
    trace!("Locked unit: {}", id_to_start);
    let name = unit_locked.conf.name();

    let status_table_locked = run_info.status_table.read().unwrap();

    // if not all dependencies are yet started ignore this call. This unit will be activated again when
    // the next dependency gets ready
    let all_deps_ready = unit_locked.install.after.iter().fold(true, |acc, elem| {
        let is_started = {
            let status = status_table_locked.get(elem).unwrap();
            let status_locked = status.lock().unwrap();
            *status_locked == UnitStatus::Started
                || *status_locked == UnitStatus::StartedWaitingForSocket
        };
        acc && is_started
    });
    if !all_deps_ready {
        trace!(
            "Unit: {} ignores activation. Not all dependencies have been started (needs: {:?})",
            unit_locked.conf.name(),
            unit_locked.install.after,
        );
        return Ok(StartResult::Ignored);
    }

    // Check if the unit is currently starting. Update the status to starting if not
    {
        let status = status_table_locked.get(&id_to_start).unwrap();
        trace!("Lock status for: {}", name);
        let mut status_locked = status.lock().unwrap();
        trace!("Locked status for: {}", name);

        // if status is already on Started then allow ignore must be false. This happens when socket activation is happening
        // TODO make this relation less weird. Maybe add a separate code path for socket activation
        let wait_for_socket_act = *status_locked == UnitStatus::Started && allow_ignore;
        let needs_intial_run =
            *status_locked == UnitStatus::NeverStarted || *status_locked == UnitStatus::Stopped;
        if wait_for_socket_act && !needs_intial_run {
            trace!(
                "Don't activate Unit: {:?}. Has status: {:?}",
                unit_locked.conf.name(),
                *status_locked
            );
            return Ok(StartResult::Ignored);
        }
        if needs_intial_run {
            *status_locked = UnitStatus::Starting;
        }
    }
    let next_services_ids = unit_locked.install.before.clone();

    unit_locked
        .activate(
            run_info.pid_table.clone(),
            run_info.fd_store.clone(),
            notification_socket_path.clone(),
            &eventfds,
            allow_ignore,
        )
        .map(|new_status| {
            // Update the status while we still lock the unit
            let status_table_locked = run_info.status_table.read().unwrap();
            let status = status_table_locked.get(&unit_locked.id).unwrap();
            let mut status_locked = status.lock().unwrap();
            *status_locked = new_status;
            StartResult::Started(next_services_ids)
        })
        .map_err(|e| {
            format!(
                "Error while starting unit {}: {}",
                unit_locked.conf.name(),
                e
            )
        })
    // drop all the locks "at once". Ordering of dropping should be irrelevant?
}

pub fn activate_units(
    run_info: ArcRuntimeInfo,
    notification_socket_path: std::path::PathBuf,
    eventfds: Vec<EventFd>,
) {
    let mut root_units = Vec::new();

    for (id, unit) in &*run_info.unit_table.read().unwrap() {
        let unit_locked = unit.lock().unwrap();
        if unit_locked.install.after.is_empty() {
            root_units.push(*id);
            trace!("Root unit: {}", unit_locked.conf.name());
        }
    }

    let tpool = ThreadPool::new(6);
    let eventfds_arc = Arc::new(eventfds);
    activate_units_recursive(
        root_units,
        run_info,
        tpool.clone(),
        notification_socket_path,
        eventfds_arc,
    );

    tpool.join();
}
