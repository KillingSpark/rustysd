//! Activate units (recursively and parallel along the dependency tree)

use crate::runtime_info::*;
use crate::services::ServiceErrorReason;
use crate::units::*;

use log::{error, trace};
use std::sync::{Arc, Mutex};
use threadpool::ThreadPool;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct UnitOperationError {
    pub reason: UnitOperationErrorReason,
    pub unit_name: String,
    pub unit_id: UnitId,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum UnitOperationErrorReason {
    GenericStartError(String),
    GenericStopError(String),
    SocketOpenError(String),
    SocketCloseError(String),
    ServiceStartError(ServiceErrorReason),
    ServiceStopError(ServiceErrorReason),
    DependencyError(Vec<UnitId>),
}

impl std::fmt::Display for UnitOperationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match &self.reason {
            UnitOperationErrorReason::GenericStartError(msg) => {
                write!(
                    f,
                    "Unit {} (ID {}) failed to start because: {}",
                    self.unit_name, self.unit_id, msg
                )?;
            }
            UnitOperationErrorReason::GenericStopError(msg) => {
                write!(
                    f,
                    "Unit {} (ID {}) failed to stop cleanly because: {}",
                    self.unit_name, self.unit_id, msg
                )?;
            }
            UnitOperationErrorReason::ServiceStartError(msg) => {
                write!(
                    f,
                    "Service {} (ID {}) failed to start because: {}",
                    self.unit_name, self.unit_id, msg
                )?;
            }
            UnitOperationErrorReason::ServiceStopError(msg) => {
                write!(
                    f,
                    "Service {} (ID {}) failed to stop cleanly because: {}",
                    self.unit_name, self.unit_id, msg
                )?;
            }
            UnitOperationErrorReason::SocketOpenError(msg) => {
                write!(
                    f,
                    "Socket {} (ID {}) failed to open because: {}",
                    self.unit_name, self.unit_id, msg
                )?;
            }
            UnitOperationErrorReason::SocketCloseError(msg) => {
                write!(
                    f,
                    "Socket {} (ID {}) failed to close cleanly because: {}",
                    self.unit_name, self.unit_id, msg
                )?;
            }
            UnitOperationErrorReason::DependencyError(ids) => {
                write!(
                    f,
                    "The unit {} (ID {}) failed to start/stop because these related units did not have the expected state: {:?}",
                    self.unit_name, self.unit_id, ids
                )?;
            }
        }
        Ok(())
    }
}

pub fn unstarted_deps(id: &UnitId, run_info: &RuntimeInfo) -> Vec<UnitId> {
    let unit = match run_info.unit_table.get(id) {
        Some(unit) => unit,
        None => {
            // If this occurs, there is a flaw in the handling of dependencies
            // IDs should be purged globally when units get removed
            return vec![];
        }
    };

    // if not all dependencies are yet started ignore this call. This unit will be activated again when
    // the next dependency gets ready
    let unstarted_deps = unit
        .common
        .dependencies
        .after
        .iter()
        .fold(Vec::new(), |mut acc, elem| {
            let required = unit.common.dependencies.requires.contains(elem);
            let elem_unit = run_info.unit_table.get(elem).unwrap();
            let status_locked = elem_unit.common.status.read().unwrap();
            let ready = if required {
                status_locked.is_started()
            } else {
                *status_locked != UnitStatus::NeverStarted
            };

            if !ready {
                acc.push(elem.clone());
            }
            acc
        });
    unstarted_deps
}

#[derive(Debug)]
pub enum StartResult {
    Started(Vec<UnitId>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivationSource {
    Regular,
    SocketActivation,
}

impl ActivationSource {
    pub fn is_socket_activation(&self) -> bool {
        match self {
            ActivationSource::SocketActivation => true,
            _ => false,
        }
    }
}

/// Activate the unit and return all units that are ordered later than this unit
///
/// This also checks that all 'requires' relations are held up
pub fn activate_unit(
    id_to_start: UnitId,
    run_info: &RuntimeInfo,
    source: ActivationSource,
) -> std::result::Result<StartResult, UnitOperationError> {
    trace!("Activate id: {:?}", id_to_start);

    let unit = match run_info.unit_table.get(&id_to_start) {
        Some(unit) => unit,
        None => {
            // If this occurs, there is a flaw in the handling of dependencies
            // IDs should be purged globally when units get removed
            return Err(UnitOperationError {
                reason: UnitOperationErrorReason::GenericStartError(
                    "Tried to activate a unit that can not be found".into(),
                ),
                unit_name: id_to_start.name.clone(),
                unit_id: id_to_start.clone(),
            });
        }
    };

    let next_services_ids = unit.common.dependencies.before.clone();

    unit.activate(run_info.clone(), source)
        .map(|_| StartResult::Started(next_services_ids))
}

/// Walk the unit graph and find all units that need to be started to be able to start all units in ids_to_start.
///
/// This extends the ids_to_start with the additional ids
pub fn collect_unit_start_subgraph(ids_to_start: &mut Vec<UnitId>, unit_table: &UnitTable) {
    // iterate until the set-size doesnt change anymore. This works because there is only a finite set of units that can be added here.
    // This requires that ids only appear once in the set
    loop {
        let mut new_ids = Vec::new();
        for id in ids_to_start.iter() {
            let unit = unit_table.get(id).unwrap();
            new_ids.extend(unit.common.dependencies.start_before_this());
            new_ids.extend(unit.common.dependencies.start_concurrently_with_this());
        }
        new_ids.sort();
        new_ids.dedup();
        new_ids = new_ids
            .into_iter()
            .filter(|id| !ids_to_start.contains(id))
            .collect();

        if new_ids.len() == 0 {
            break;
        } else {
            ids_to_start.extend(new_ids);
        }
    }
}

/// Collects the subgraph of units that need to be started to reach the target_id (Note: not required to be a unit of type .target).
/// Then starts these units as concurrently as possible respecting the before <-> after ordering
pub fn activate_needed_units(
    target_id: UnitId,
    run_info: ArcMutRuntimeInfo,
) -> Vec<UnitOperationError> {
    let mut needed_ids = vec![target_id.clone()];
    {
        let run_info = run_info.read().unwrap();
        collect_unit_start_subgraph(&mut needed_ids, &run_info.unit_table);
    }
    trace!("Needed units to start {:?}: {:?}", target_id, needed_ids);

    // collect all 'root' units. These are units that do not have any 'after' relations to other unstarted units.
    // These can be started and the the graph can be traversed and other units can be started as soon as
    // all other units they depend on are started. This works because the units form an DAG if only
    // the 'after' relations are considered for traversal.
    let root_units = { find_startable_units(&needed_ids, &*run_info.read().unwrap()) };
    trace!("Root units found: {:?}", root_units);

    // TODO make configurable or at least make guess about amount of threads
    let tpool = ThreadPool::new(6);
    let errors = Arc::new(Mutex::new(Vec::new()));
    activate_units_recursive(
        root_units,
        Arc::new(needed_ids),
        run_info,
        tpool.clone(),
        errors.clone(),
    );

    tpool.join();
    // TODO can we handle errors in a more meaningful way?
    let errs = (&*errors.lock().unwrap()).clone();
    for err in &errs {
        error!("Error while activating unit graph: {}", err);
    }
    errs
}

/// Check for all units in this Vec, if all units this depends on are running
fn find_startable_units(ids: &Vec<UnitId>, run_info: &RuntimeInfo) -> Vec<UnitId> {
    let mut startable = Vec::new();

    for id in ids {
        if unstarted_deps(id, run_info).is_empty() {
            startable.push(id.clone());
        }
    }
    startable
}

/// Start all units in ids_to_start and push jobs into the threadpool to start all following units.
///
/// Only do so for the units in filter_ids
fn activate_units_recursive(
    ids_to_start: Vec<UnitId>,
    filter_ids: Arc<Vec<UnitId>>,
    run_info: ArcMutRuntimeInfo,
    tpool: ThreadPool,
    errors: Arc<Mutex<Vec<UnitOperationError>>>,
) {
    let startables = { find_startable_units(&ids_to_start, &*run_info.read().unwrap()) };
    let startables: Vec<UnitId> = startables
        .into_iter()
        .filter(|id| filter_ids.contains(&id))
        .collect();

    for id in startables {
        // make copies to move into the closure
        let run_info_copy = run_info.clone();
        let tpool_copy = tpool.clone();
        let errors_copy = errors.clone();
        let filter_ids_copy = filter_ids.clone();
        tpool.execute(move || {
            match activate_unit(
                id,
                &*run_info_copy.read().unwrap(),
                ActivationSource::Regular,
            ) {
                Ok(StartResult::Started(next_services_ids)) => {
                    // make copies to move into the closure
                    let run_info_copy2 = run_info_copy.clone();
                    let tpool_copy2 = tpool_copy.clone();
                    let errors_copy2 = errors_copy.clone();
                    let filter_ids_copy2 = filter_ids_copy.clone();

                    let next_services_job = move || {
                        activate_units_recursive(
                            next_services_ids,
                            filter_ids_copy2,
                            run_info_copy2,
                            tpool_copy2,
                            errors_copy2,
                        );
                    };
                    tpool_copy.execute(next_services_job);
                }
                Err(e) => {
                    if let UnitOperationErrorReason::DependencyError(_) = e.reason {
                        // Thats ok. The unit is waiting for more dependencies and will be
                        // activated again when another dependency has finished starting

                        // This should not happen though, since we filter the units beforehand
                        // to only get the startables
                    } else {
                        error!("Error while activating unit {}", e);
                        errors_copy.lock().unwrap().push(e);
                    }
                }
            }
        });
    }
}
