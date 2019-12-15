use super::start_service::*;
use std::collections::HashMap;
use std::error::Error;
use std::os::unix::io::RawFd;
use std::os::unix::net::UnixDatagram;
use std::process::{Command, Stdio};
use std::sync::Arc;
use threadpool::ThreadPool;

use crate::units::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ServiceStatus {
    NeverRan,
    Starting,
    Running,
    Stopped,
}

impl ToString for ServiceStatus {
    fn to_string(&self) -> String {
        match *self {
            ServiceStatus::NeverRan => "NeverRan".into(),
            ServiceStatus::Running => "Running".into(),
            ServiceStatus::Starting => "Starting".into(),
            ServiceStatus::Stopped => "Stopped".into(),
        }
    }
}

#[derive(Debug)]
pub struct ServiceRuntimeInfo {
    pub restarted: u64,
    pub up_since: Option<std::time::Instant>,
}

#[derive(Debug)]
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
    service_table: ArcMutServiceTable,
    pid_table: ArcMutPidTable,
    sockets: ArcMutSocketTable,
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
                        service_table.lock().unwrap().get(id).unwrap().conf.name(),
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

    let mut service_table_locked = service_table.lock().unwrap();
    let service_table_locked: &mut HashMap<_, _> = &mut service_table_locked;
    let unit = service_table_locked.get_mut(&srvc_id).unwrap();
    if let UnitSpecialized::Service(srvc) = &mut unit.specialized {
        pid_table_locked.remove(&pid);
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
    tpool: ThreadPool,
    notification_socket_path: std::path::PathBuf,
    eventfds: Arc<Vec<RawFd>>,
) {
    for id in ids_to_start {
        let tpool_copy = ThreadPool::clone(&tpool);
        let sockets_copy_next_jobs = Arc::clone(&sockets);
        let services_copy_next_jobs = Arc::clone(&services);
        let pids_copy_next_jobs = Arc::clone(&pids);
        let notification_socket_path_copy_next_jobs = notification_socket_path.clone();
        let eventfds_next_jobs = eventfds.clone();

        let pids_copy_this_job = Arc::clone(&pids);
        let services_copy_this_job = Arc::clone(&services);
        let sockets_copy_this_job = Arc::clone(&sockets);
        let notification_socket_path_copy_this_job = notification_socket_path.clone();
        let eventfds_this_job = eventfds.clone();

        let mut unit = {
            let mut services_locked = services.lock().unwrap();
            services_locked.remove(&id).unwrap()
        };
        let next_services_ids = unit.install.before.clone();
        let start_synchron = if let UnitSpecialized::Service(srvc) = &unit.specialized {
            srvc.socket_names.is_empty()
        } else {
            false
        };

        trace!(
            "Start service {} synchron: {}",
            unit.conf.name(),
            start_synchron
        );

        let this_service_job = move || {
            let name = unit.conf.name();
            if let UnitSpecialized::Service(srvc) = &mut unit.specialized {
                match srvc.status {
                    ServiceStatus::NeverRan => {
                        start_service(
                            srvc,
                            name,
                            sockets_copy_this_job,
                            notification_socket_path_copy_this_job,
                        );
                        if let Some(new_pid) = srvc.pid {
                            {
                                let mut services_locked = services_copy_this_job.lock().unwrap();
                                services_locked.insert(id, unit)
                            };
                            {
                                let mut pids = pids_copy_this_job.lock().unwrap();
                                pids.insert(new_pid, PidEntry::Service(id));
                            }
                            crate::notification_handler::notify_event_fds(&eventfds_this_job)
                        } else {
                            // TODO dont even start services that require this one
                        }
                    }
                    _ => unreachable!(),
                }
            }
        };

        if start_synchron {
            this_service_job();
        } else {
            tpool.execute(this_service_job);
        }

        let next_services_job = move || {
            run_services_recursive(
                next_services_ids,
                Arc::clone(&services_copy_next_jobs),
                Arc::clone(&pids_copy_next_jobs),
                Arc::clone(&sockets_copy_next_jobs),
                ThreadPool::clone(&tpool_copy),
                notification_socket_path_copy_next_jobs,
                eventfds_next_jobs,
            );
        };

        tpool.execute(next_services_job);
    }
}

pub fn run_services(
    services: ArcMutServiceTable,
    sockets: ArcMutSocketTable,
    notification_socket_path: std::path::PathBuf,
    eventfds: Vec<RawFd>,
) -> ArcMutPidTable {
    let pids = HashMap::new();
    let mut root_services = Vec::new();

    for (id, unit) in &*services.lock().unwrap() {
        if unit.install.after.is_empty() {
            root_services.push(*id);
            trace!("Root service: {}", unit.conf.name());
        }
    }

    let tpool = ThreadPool::new(6);
    let pids_arc = Arc::new(Mutex::new(pids));
    let eventfds_arc = Arc::new(eventfds);
    run_services_recursive(
        root_services,
        Arc::clone(&services),
        Arc::clone(&pids_arc),
        Arc::clone(&sockets),
        tpool.clone(),
        notification_socket_path,
        eventfds_arc,
    );

    tpool.join();

    pids_arc
}
