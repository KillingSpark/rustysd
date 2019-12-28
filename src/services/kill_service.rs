use crate::platform::EventFd;
use crate::units::*;
use std::sync::Arc;

pub fn kill_service(id_to_kill: UnitId, run_info: ArcRuntimeInfo) {
    let unit_table_locked = run_info.unit_table.read().unwrap();
    let mut pid_table_locked = run_info.pid_table.lock().unwrap();
    let srvc_unit = unit_table_locked.get(&id_to_kill).unwrap();
    let mut unit_locked = srvc_unit.lock().unwrap();
    let srvc_id = unit_locked.id;
    let srvc_name = unit_locked.conf.name();

    if let UnitSpecialized::Service(srvc) = &mut unit_locked.specialized {
        {
            let status_table_locked = run_info.status_table.read().unwrap();
            let status = status_table_locked.get(&id_to_kill).unwrap();
            let mut status_locked = status.lock().unwrap();
            *status_locked = UnitStatus::Stopping;
        }
        srvc.kill(srvc_id, &srvc_name, &mut *pid_table_locked);
        {
            let status_table_locked = run_info.status_table.read().unwrap();
            let status = status_table_locked.get(&id_to_kill).unwrap();
            let mut status_locked = status.lock().unwrap();
            *status_locked = UnitStatus::Stopped;
        }
    }
}

pub fn kill_services(ids_to_kill: Vec<UnitId>, run_info: ArcRuntimeInfo) {
    //TODO killall services that require this service
    for id in ids_to_kill {
        kill_service(id, run_info.clone());
    }
}

pub fn restart_service(
    id_to_restart: UnitId,
    run_info: ArcRuntimeInfo,
    notification_socket_path: std::path::PathBuf,
    eventfds: Arc<Vec<EventFd>>,
) -> std::result::Result<(), std::string::String> {
    kill_service(id_to_restart, run_info.clone());
    crate::units::activate_unit(
        id_to_restart,
        run_info,
        notification_socket_path,
        eventfds,
        true,
    )
    .map(|_| ())
}
