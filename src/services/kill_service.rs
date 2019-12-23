use crate::platform::EventFd;
use crate::units::*;
use std::sync::Arc;

pub fn kill_service(
    id_to_kill: InternalId,
    unit_table: ArcMutUnitTable,
    pid_table: ArcMutPidTable,
) {
    let unit_table_locked = unit_table.read().unwrap();
    let mut pid_table_locked = pid_table.lock().unwrap();
    let srvc_unit = unit_table_locked.get(&id_to_kill).unwrap();
    let mut unit_locked = srvc_unit.lock().unwrap();
    let srvc_id = unit_locked.id;
    let srvc_name = unit_locked.conf.name();
    if let UnitSpecialized::Service(srvc) = &mut unit_locked.specialized {
        srvc.kill(srvc_id, &srvc_name, &mut *pid_table_locked);
    }
}

pub fn kill_services(
    ids_to_kill: Vec<InternalId>,
    unit_table: ArcMutUnitTable,
    pid_table: ArcMutPidTable,
) {
    //TODO killall services that require this service
    for id in ids_to_kill {
        kill_service(id, unit_table.clone(), pid_table.clone());
    }
}

pub fn restart_service(
    id_to_restart: InternalId,
    unit_table: ArcMutUnitTable,
    pid_table: ArcMutPidTable,
    notification_socket_path: std::path::PathBuf,
    eventfds: Arc<Vec<EventFd>>,
) -> std::result::Result<(), std::string::String> {
    kill_service(id_to_restart, unit_table.clone(), pid_table.clone());
    crate::units::activate_unit(
        id_to_restart,
        None,
        unit_table,
        pid_table,
        notification_socket_path,
        eventfds,
        true,
    )
    .map(|_| ())
}
