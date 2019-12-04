use crate::start_service::*;
use std::collections::HashMap;
use std::error::Error;
use std::os::unix::net::UnixDatagram;
use std::process::{Command, Stdio};
use std::sync::Arc;
use threadpool::ThreadPool;

use crate::units::*;

#[derive(Clone)]
pub enum ServiceStatus {
    NeverRan,
    Starting,
    Running,
    Stopped,
}

#[derive(Clone)]
pub struct ServiceRuntimeInfo {
    pub restarted: u64,
    pub up_since: Option<std::time::Instant>,
}

#[derive(Clone)]
pub struct Service {
    pub pid: Option<nix::unistd::Pid>,
    pub service_config: Option<ServiceConfig>,

    pub status: ServiceStatus,
    pub socket_names: Vec<String>,

    pub status_msgs: Vec<String>,

    pub runtime_info: ServiceRuntimeInfo,

    pub notifications: Option<Arc<Mutex<UnixDatagram>>>,
    pub notifications_buffer: String,
}

pub fn kill_services(
    ids_to_kill: Vec<InternalId>,
    service_table: &mut ServiceTable,
    pid_table: &mut PidTable,
) {
    //TODO killall services that require this service
    for id in ids_to_kill {
        let srvc_unit = service_table.get_mut(&id).unwrap();
        if let UnitSpecialized::Service(srvc) = &srvc_unit.specialized {
            let split: Vec<&str> = match &srvc.service_config {
                Some(conf) => {
                    if conf.stop.is_empty() {
                        continue;
                    }
                    conf.stop.split(' ').collect()
                }
                None => continue,
            };

            let mut cmd = Command::new(split[0]);
            for part in &split[1..] {
                cmd.arg(part);
            }
            cmd.stdout(Stdio::null());

            match cmd.spawn() {
                Ok(child) => {
                    pid_table.insert(
                        nix::unistd::Pid::from_raw(child.id() as i32),
                        PidEntry::Stop(srvc_unit.id),
                    );
                    trace!(
                        "Stopped Service: {} with pid: {:?}",
                        srvc_unit.conf.name(),
                        srvc.pid
                    );
                }
                Err(e) => panic!(e.description().to_owned()),
            }
        }
    }
}

pub fn service_exit_handler(
    pid: i32,
    code: i32,
    service_table: ArcMutServiceTable,
    pid_table: ArcMutPidTable,
    sockets: ArcMutSocketTable,
    notification_socket_path: std::path::PathBuf,
) {
    let pid_table_locked = &mut *pid_table.lock().unwrap();
    let srvc_id = {
        *(match pid_table_locked.get(&nix::unistd::Pid::from_raw(pid)) {
            Some(entry) => match entry {
                PidEntry::Service(id) => id,
                PidEntry::Stop(id) => {
                    trace!(
                        "Stop process for service: {} exited with code: {}",
                        service_table.lock().unwrap().get(id).unwrap().conf.name(),
                        code
                    );
                    pid_table_locked.remove(&nix::unistd::Pid::from_raw(pid));
                    return;
                }
            },
            None => {
                unreachable!("All spawned processes should have a pid entry");
            }
        })
    };

    trace!(
        "Service with id: {} pid: {} exited with code: {}",
        srvc_id,
        pid,
        code
    );

    let mut service_table_locked = service_table.lock().unwrap();
    let service_table_locked: &mut HashMap<_, _> = &mut service_table_locked;
    let unit = service_table_locked.get_mut(&srvc_id).unwrap();
    if let UnitSpecialized::Service(srvc) = &mut unit.specialized {
        pid_table_locked.remove(&(nix::unistd::Pid::from_raw(pid)));
        srvc.status = ServiceStatus::Stopped;

        if let Some(conf) = &srvc.service_config {
            if conf.keep_alive {
                start_service(srvc, unit.conf.name(), sockets, notification_socket_path);
                if let Some(pid) = srvc.pid {
                    srvc.runtime_info.restarted += 1;
                    pid_table_locked.insert(pid, PidEntry::Service(unit.id));
                }
            } else {
                trace!(
                    "Killing all services requiring service with id {}: {:?}",
                    srvc_id,
                    unit.install.required_by
                );
                kill_services(
                    unit.install.required_by.clone(),
                    service_table_locked,
                    pid_table_locked,
                );
            }
        }
    }
}

use std::sync::Mutex;
fn run_services_recursive(
    ids_to_start: Vec<InternalId>,
    services: ArcMutServiceTable,
    pids: ArcMutPidTable,
    sockets: ArcMutSocketTable,
    tpool: Arc<Mutex<ThreadPool>>,
    waitgroup: crossbeam::sync::WaitGroup,
    notification_socket_path: std::path::PathBuf,
) {
    for id in ids_to_start {
        let waitgroup_copy = waitgroup.clone();
        let tpool_copy = Arc::clone(&tpool);
        let services_copy = Arc::clone(&services);
        let pids_copy = Arc::clone(&pids);
        let sockets_copy = Arc::clone(&sockets);
        let notification_socket_path_copy = notification_socket_path.clone();

        let job = move || {
            let mut unit = {
                let mut services_locked = services_copy.lock().unwrap();
                services_locked.get_mut(&id).unwrap().clone()
            };
            let name = unit.conf.name();
            if let UnitSpecialized::Service(srvc) = &mut unit.specialized {
                match srvc.status {
                    ServiceStatus::NeverRan => {
                        start_service(
                            srvc,
                            name.clone(),
                            sockets_copy.clone(),
                            notification_socket_path_copy.clone(),
                        );
                        if let Some(new_pid) = srvc.pid {
                            {
                                let mut services_locked = services_copy.lock().unwrap();
                                services_locked.insert(id, unit.clone()).unwrap().clone()
                            };
                            {
                                let mut pids = pids_copy.lock().unwrap();
                                pids.insert(new_pid, PidEntry::Service(unit.id));
                            }
                        } else {
                            // TODO dont event start services that require this one
                        }
                    }
                    _ => unreachable!(),
                }

                run_services_recursive(
                    unit.install.before.clone(),
                    Arc::clone(&services_copy),
                    Arc::clone(&pids_copy),
                    Arc::clone(&sockets_copy),
                    Arc::clone(&tpool_copy),
                    waitgroup_copy,
                    notification_socket_path_copy,
                );
            }
        };

        {
            let tpool_locked = tpool.lock().unwrap();
            tpool_locked.execute(job);
        }
    }
    drop(waitgroup);
}

pub fn run_services(
    services: ServiceTable,
    sockets: SocketTable,
    notification_socket_path: std::path::PathBuf,
) -> (ArcMutServiceTable, ArcMutSocketTable, ArcMutPidTable) {
    let pids = HashMap::new();
    let mut root_services = Vec::new();

    for (id, unit) in &services {
        if unit.install.after.is_empty() {
            root_services.push(*id);
            trace!("Root service: {}", unit.conf.name());
        }
    }

    let pool_arc = Arc::new(Mutex::new(ThreadPool::new(6)));
    let services_arc = Arc::new(Mutex::new(services));
    let pids_arc = Arc::new(Mutex::new(pids));
    let sockets_arc = Arc::new(Mutex::new(sockets));
    let waitgroup = crossbeam::sync::WaitGroup::new();
    run_services_recursive(
        root_services,
        Arc::clone(&services_arc),
        Arc::clone(&pids_arc),
        sockets_arc.clone(),
        pool_arc,
        waitgroup.clone(),
        notification_socket_path,
    );

    waitgroup.wait();

    (services_arc, sockets_arc, pids_arc)
}
