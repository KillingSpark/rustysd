use crate::runtime_info::UnitTable;
use crate::units::*;
use std::collections::HashMap;

use std::sync::{RwLockReadGuard, RwLockWriteGuard};

/// This is a helper function to lock a set of units either read or write
///
/// This is needed to be able to lock a unit exclusively and all related units shared so we can uphold
/// invariants while running an operation on the exclusively locked unit.
pub fn aquire_locks<'table>(
    mut lock_exclusive: Vec<UnitId>,
    mut lock_shared: Vec<UnitId>,
    unit_table: &'table UnitTable,
) -> (
    HashMap<UnitId, RwLockWriteGuard<'table, UnitStatus>>,
    HashMap<UnitId, RwLockReadGuard<'table, UnitStatus>>,
) {
    let mut exclusive = HashMap::new();
    let mut shared = HashMap::new();

    lock_exclusive.sort();
    lock_exclusive.dedup();
    lock_shared.sort();
    lock_shared.dedup();
    if lock_exclusive.iter().any(|id| lock_shared.contains(id)) {
        panic!("Cant lock shared and exclusive at the same time!");
    }

    while !lock_shared.is_empty() && !lock_exclusive.is_empty() {
        if &lock_exclusive.last().unwrap() < &lock_shared.last().unwrap() {
            let id = lock_exclusive.remove(lock_exclusive.len() - 1);
            let unit = unit_table.get(&id).unwrap();
            let locked_status = unit.common.status.write().unwrap();
            exclusive.insert(id, locked_status);
        } else {
            let id = lock_shared.remove(lock_shared.len() - 1);
            let unit = unit_table.get(&id).unwrap();
            let locked_status = unit.common.status.read().unwrap();
            shared.insert(id, locked_status);
        }
    }

    lock_shared.reverse();
    lock_exclusive.reverse();
    for id in lock_shared {
        let unit = unit_table.get(&id).unwrap();
        let locked_status = unit.common.status.read().unwrap();
        shared.insert(id, locked_status);
    }
    for id in lock_exclusive {
        let unit = unit_table.get(&id).unwrap();
        let locked_status = unit.common.status.write().unwrap();
        exclusive.insert(id, locked_status);
    }

    (exclusive, shared)
}
