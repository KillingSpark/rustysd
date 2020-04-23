//! Activate units (recursively and parallel along the dependency tree)

use crate::runtime_info::*;
use crate::services::ServiceErrorReason;
use crate::units::*;

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
        }
        Ok(())
    }
}

fn activate_units_recursive(
    ids_to_start: Vec<UnitId>,
    run_info: ArcMutRuntimeInfo,
    tpool: ThreadPool,
    errors: Arc<Mutex<Vec<UnitOperationError>>>,
) {
    for id in ids_to_start {
        let run_info_copy = run_info.clone();
        let tpool_copy = tpool.clone();
        let errors_copy = errors.clone();
        tpool.execute(move || {
            let run_info_copy2 = run_info_copy.clone();
            let tpool_copy2 = tpool_copy.clone();
            let errors_copy2 = errors_copy.clone();

            match activate_unit(
                id,
                &*run_info_copy.read().unwrap(),
                ActivationSource::Regular,
            ) {
                Ok(StartResult::Started(next_services_ids)) => {
                    let next_services_job = move || {
                        activate_units_recursive(
                            next_services_ids,
                            run_info_copy2,
                            tpool_copy2,
                            errors_copy2,
                        );
                    };
                    tpool_copy.execute(next_services_job);
                }
                Ok(StartResult::WaitForDependencies(_)) => {
                    // Thats ok. The unit is waiting for more dependencies and will be
                    // activated again when another dependency has finished starting
                }
                Err(e) => {
                    error!("Error while activating unit {}", e);
                    errors_copy.lock().unwrap().push(e);
                }
            }
        });
    }
}

#[derive(Debug)]
pub enum StartResult {
    Started(Vec<UnitId>),
    WaitForDependencies(Vec<UnitId>),
}

#[derive(Debug, Clone, Copy)]
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
    if !unstarted_deps.is_empty() {
        trace!(
            "Unit: {} ignores activation. Not all dependencies have been started (still waiting for: {:?})",
            unit.id.name,
            unstarted_deps,
        );
        return Ok(StartResult::WaitForDependencies(unstarted_deps));
    }

    let next_services_ids = unit.common.dependencies.before.clone();

    unit.activate(run_info.clone(), source)
        .map(|_| StartResult::Started(next_services_ids))
}

pub fn activate_units(run_info: ArcMutRuntimeInfo) {
    // collect all 'root' units. These are units that do not have any 'after' relations to other units.
    // These can be started and the the tree can be traversed and other units can be started as soon as
    // all other units they depend on are started. This works because the units form an DAG if one only
    // uses the 'after' relations.
    let mut root_units = Vec::new();

    {
        let run_info = &run_info.read().unwrap();
        trace!("Collect root units");
        for (id, unit) in run_info.unit_table.iter() {
            if unit.common.dependencies.after.is_empty() {
                root_units.push(id.clone());
                trace!("Root unit: {}", unit.id.name);
            } else {
                trace!(
                    "Not a root unit: {}. Starts after: {:?}",
                    unit.id.name,
                    unit.common.dependencies.after
                );
            }
        }
    }

    // TODO make configurable or at least make guess about amount of threads
    let tpool = ThreadPool::new(6);
    let errors = Arc::new(Mutex::new(Vec::new()));
    activate_units_recursive(root_units, run_info, tpool.clone(), errors.clone());

    tpool.join();
    // TODO can we handle errors in a more meaningful way?
    for err in &*errors.lock().unwrap() {
        error!("{}", err);
    }
}
