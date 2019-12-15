use std::collections::HashMap;
use std::os::unix::io::RawFd;
use std::sync::Arc;
use std::sync::Mutex;
use threadpool::ThreadPool;
use super::units::*;

fn activate_units_recursive(
    ids_to_start: Vec<InternalId>,
    started_ids: Arc<Mutex<Vec<InternalId>>>,
    unit_table: ArcMutUnitTable,
    pids: ArcMutPidTable,
    tpool: ThreadPool,
    notification_socket_path: std::path::PathBuf,
    eventfds: Arc<Vec<RawFd>>,
) {
    for id in ids_to_start {
        let tpool_copy = ThreadPool::clone(&tpool);
        let services_copy_next_jobs = Arc::clone(&unit_table);
        let pids_copy_next_jobs = Arc::clone(&pids);
        let notification_socket_path_copy_next_jobs = notification_socket_path.clone();
        let eventfds_next_jobs = eventfds.clone();
        let started_ids_copy = started_ids.clone();

        let mut unit = {
            let mut services_locked = unit_table.lock().unwrap();
            let unit = services_locked.remove(&id).unwrap();
            let started_ids_locked = started_ids.lock().unwrap();

            // if not all dependencies are yet started ignore this call. THis unit will be activated again when
            // the next dependency gets ready
            let all_deps_ready = unit
                .install
                .after
                .iter()
                .fold(true, |acc, elem| acc && started_ids_locked.contains(elem));
            if !all_deps_ready {
                services_locked.insert(id, unit);
                return;
            }
            unit
        };
        let next_services_ids = unit.install.before.clone();

        match unit.activate(
            unit_table.clone(),
            pids.clone(),
            notification_socket_path.clone(),
            &eventfds,
        ) {
            Ok(_) => {
                let mut started_ids_locked = started_ids.lock().unwrap();
                started_ids_locked.push(id);
                let next_services_job = move || {
                    activate_units_recursive(
                        next_services_ids,
                        started_ids_copy,
                        services_copy_next_jobs,
                        pids_copy_next_jobs,
                        ThreadPool::clone(&tpool_copy),
                        notification_socket_path_copy_next_jobs,
                        eventfds_next_jobs,
                    );
                };
                tpool.execute(next_services_job);
            }
            Err(e) => error!("Error while activating unit {}: {}", unit.conf.name(), e),
        }

        {
            let mut services_locked = unit_table.lock().unwrap();
            services_locked.insert(id, unit)
        };
    }
}

pub fn activate_units(
    unit_table: ArcMutUnitTable,
    notification_socket_path: std::path::PathBuf,
    eventfds: Vec<RawFd>,
) -> ArcMutPidTable {
    let pids = HashMap::new();
    let mut root_units = Vec::new();

    for (id, unit) in &*unit_table.lock().unwrap() {
        if unit.install.after.is_empty() {
            root_units.push(*id);
            trace!("Root unit: {}", unit.conf.name());
        }
    }

    let tpool = ThreadPool::new(6);
    let pids_arc = Arc::new(Mutex::new(pids));
    let eventfds_arc = Arc::new(eventfds);
    let started_ids = Arc::new(Mutex::new(Vec::new()));
    activate_units_recursive(
        root_units,
        started_ids,
        Arc::clone(&unit_table),
        Arc::clone(&pids_arc),
        tpool.clone(),
        notification_socket_path,
        eventfds_arc,
    );

    tpool.join();

    pids_arc
}
