use crate::fd_store::FDStore;
use crate::units::*;

use nix::unistd::Pid;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

pub type UnitTable = HashMap<UnitId, Unit>;
pub type PidTable = HashMap<Pid, PidEntry>;
pub type MutFDStore = RwLock<FDStore>;

pub struct RuntimeInfo {
    pub unit_table: UnitTable,
    pub pid_table: Mutex<PidTable>,
    pub fd_store: MutFDStore,
    pub config: crate::config::Config,
}

// This will be passed through to all the different threads as a central state struct
pub type ArcMutRuntimeInfo = Arc<RwLock<RuntimeInfo>>;

pub fn lock_all(
    units: &mut Vec<(UnitId, Arc<Mutex<Unit>>)>,
) -> HashMap<UnitId, std::sync::MutexGuard<'_, Unit>> {
    let mut units_locked = HashMap::new();
    // sort to make sure units always get locked in the same ordering
    units.sort_by(|(lid, _), (rid, _)| lid.cmp(rid));

    for (id, unit) in units {
        trace!("Lock unit: {:?}", id);
        let other_unit_locked = unit.lock().unwrap();
        trace!("Locked unit: {:?}", id);
        units_locked.insert(id.clone(), other_unit_locked);
    }

    units_locked
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum PidEntry {
    Service(UnitId, ServiceType),
    OneshotExited(crate::signal_handler::ChildTermination),
    Helper(UnitId, String),
    HelperExited(crate::signal_handler::ChildTermination),
}
