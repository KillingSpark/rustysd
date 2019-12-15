use super::start_service::*;
use std::collections::HashMap;
use std::error::Error;
use std::os::unix::io::RawFd;
use std::os::unix::net::UnixDatagram;
use std::process::{Command, Stdio};
use std::sync::Arc;
use threadpool::ThreadPool;

use crate::units::*;

pub enum ServiceStatus {
    NeverRan,
    Starting,
    Running,
    Stopped,
}

pub struct ServiceRuntimeInfo {
    pub restarted: u64,
    pub up_since: Option<std::time::Instant>,
}

pub struct Service {
    pub pid: Option<nix::unistd::Pid>,
    pub service_config: Option<ServiceConfig>,

    pub status: ServiceStatus,
    pub socket_names: Vec<String>,

    pub status_msgs: Vec<String>,

    pub runtime_info: ServiceRuntimeInfo,

    pub notifications: Option<Arc<Mutex<UnixDatagram>>>,
    pub stdout_dup: Option<(RawFd, RawFd)>,
    pub stderr_dup: Option<(RawFd, RawFd)>,
    pub notifications_buffer: String,
}

impl Service {
    pub fn start(
        &mut self,
        id: InternalId,
        name: &String,
        sockets: ArcMutSocketTable,
        pids: ArcMutPidTable,
        notification_socket_path: std::path::PathBuf,
        eventfds: &[RawFd],
    ) {
        trace!("Start service {}", name);

        match self.status {
            ServiceStatus::NeverRan => {
                start_service(self, name.clone(), sockets, notification_socket_path);
                if let Some(new_pid) = self.pid {
                    {
                        let mut pids = pids.lock().unwrap();
                        pids.insert(new_pid, PidEntry::Service(id));
                    }
                    crate::notification_handler::notify_event_fds(&eventfds)
                } else {
                    // TODO dont even start services that require this one
                }
            }
            _ => unreachable!(),
        }
    }
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
    pid: nix::unistd::Pid,
    code: i32,
    unit_table: ArcMutServiceTable,
    pid_table: ArcMutPidTable,
    notification_socket_path: std::path::PathBuf,
) {
    let pid_table_locked = &mut *pid_table.lock().unwrap();
    let srvc_id = {
        *(match pid_table_locked.get(&pid) {
            Some(entry) => match entry {
                PidEntry::Service(id) => id,
                PidEntry::Stop(id) => {
                    trace!(
                        "Stop process for service: {} exited with code: {}",
                        unit_table.lock().unwrap().get(id).unwrap().conf.name(),
                        code
                    );
                    pid_table_locked.remove(&pid);
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

    let mut service_table_locked = unit_table.lock().unwrap();
    let service_table_locked: &mut HashMap<_, _> = &mut service_table_locked;
    let unit = service_table_locked.get_mut(&srvc_id).unwrap();
    if let UnitSpecialized::Service(srvc) = &mut unit.specialized {
        pid_table_locked.remove(&pid);
        srvc.status = ServiceStatus::Stopped;

        if let Some(conf) = &srvc.service_config {
            if conf.keep_alive {
                start_service(srvc, unit.conf.name(), unit_table.clone(), notification_socket_path);
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
            let all_deps_ready = unit.install.after.iter().fold(true, |acc, elem| acc && started_ids_locked.contains(elem));
            if !all_deps_ready{
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
                    run_services_recursive(
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

pub fn run_services(
    unit_table: ArcMutUnitTable,
    notification_socket_path: std::path::PathBuf,
    eventfds: Vec<RawFd>,
) -> ArcMutPidTable {
    let pids = HashMap::new();
    let mut root_units = Vec::new();

    for (id, unit) in &*unit_table.lock().unwrap() {
        if unit.install.after.is_empty() {
            root_units.push(*id);
            trace!("Root service: {}", unit.conf.name());
        }
    }

    let tpool = ThreadPool::new(6);
    let pids_arc = Arc::new(Mutex::new(pids));
    let eventfds_arc = Arc::new(eventfds);
    let started_ids = Arc::new(Mutex::new(Vec::new()));
    run_services_recursive(
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
