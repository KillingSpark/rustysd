use super::start_service::*;
use std::collections::HashMap;
use std::os::unix::io::RawFd;
use std::os::unix::net::UnixDatagram;
use std::sync::Arc;
use std::sync::Mutex;

use crate::sockets::Socket;
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
    pub socket_ids: Vec<InternalId>,

    pub status_msgs: Vec<String>,

    pub process_group: Option<nix::unistd::Pid>,

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
        sockets: &HashMap<InternalId, &Socket>,
        pids: ArcMutPidTable,
        notification_socket_path: std::path::PathBuf,
        eventfds: &[RawFd],
        by_socket_activation: bool,
    ) -> Result<(), String> {
        trace!("Start service {}", name);

        match self.status {
            ServiceStatus::NeverRan | ServiceStatus::Stopped => {
                if by_socket_activation || self.socket_ids.is_empty() {
                    start_service(self, name.clone(), &sockets, notification_socket_path)?;

                    if let Some(new_pid) = self.pid {
                        {
                            let mut pids = pids.lock().unwrap();
                            pids.insert(new_pid, PidEntry::Service(id));
                        }
                        crate::notification_handler::notify_event_fds(&eventfds)
                    } else {
                        // TODO dont even start services that require this one
                    }
                } else {
                    trace!(
                        "Ignore service {} start, waiting for socket activation instead",
                        name
                    );
                }
            }
            _ => error!(
                "Tried to start service {} after it was already running",
                name
            ),
        }
        Ok(())
    }
}

pub fn service_exit_handler(
    pid: nix::unistd::Pid,
    code: i32,
    unit_table: ArcMutUnitTable,
    pid_table: ArcMutPidTable,
    notification_socket_path: std::path::PathBuf,
) -> Result<(), String> {
    let srvc_id = {
        let unit_table_locked = unit_table.read().unwrap();
        let pid_table_locked = &mut *pid_table.lock().unwrap();
        *(match pid_table_locked.get(&pid) {
            Some(entry) => match entry {
                PidEntry::Service(id) => id,
                PidEntry::Stop(id) => {
                    trace!(
                        "Stop process for service: {} exited with code: {}",
                        unit_table_locked
                            .get(id)
                            .unwrap()
                            .lock()
                            .unwrap()
                            .conf
                            .name(),
                        code
                    );
                    pid_table_locked.remove(&pid);
                    return Ok(());
                }
            },
            None => {
                warn!("All spawned processes should have a pid entry");
                return Ok(());
            }
        })
    };

    // first lock
    // 1) the unit itself
    // 2) then all units this unit says it needs to be able to start (eg. socket units for services)
    // this all needs to happen under the unit_table lock because there is a deadlock
    // hazard when taking the unit_table lock after already holding a unit lock
    let mut socket_units = HashMap::new();
    let mut socket_units_locked = HashMap::new();
    let mut sockets = HashMap::new();
    let unit = {
        let units_locked = unit_table.read().unwrap();
        let unit = match units_locked.get(&srvc_id) {
            Some(unit) => Arc::clone(unit),
            None => {
                panic!("Tried to run a unit that has been removed from the map");
            }
        };
        {
            let unit_locked = unit.lock().unwrap();
            let mut socket_ids = Vec::new();
            if let UnitSpecialized::Service(srvc) = &unit_locked.specialized {
                let name = unit_locked.conf.name();
                trace!("Lock sockets for service {}", name);
                for (id, unit) in units_locked.iter() {
                    if srvc.socket_ids.contains(id) {
                        trace!("Lock unit: {}", id);
                        let unit_locked = unit.lock().unwrap();
                        trace!("Locked unit: {}", id);
                        if let UnitSpecialized::Socket(sock) = &unit_locked.specialized {
                            socket_ids.push((unit_locked.id, sock.name.clone()));
                            socket_units.insert(*id, Arc::clone(unit));
                        }
                    }
                }
                for (id, unit) in &socket_units {
                    let unit_locked = unit.lock().unwrap();
                    socket_units_locked.insert(*id, unit_locked);
                }
                for (id, unit_locked) in &socket_units_locked {
                    if let UnitSpecialized::Socket(sock) = &unit_locked.specialized {
                        sockets.insert(*id, sock);
                    }
                }
                trace!("Done locking sockets for service {}", name);
            }
        }
        unit
    };
    let unit_locked = &mut *unit.lock().unwrap();

    trace!(
        "Service with id: {}, name: {} pid: {} exited with code: {}",
        srvc_id,
        unit_locked.conf.name(),
        pid,
        code
    );

    if let UnitSpecialized::Service(srvc) = &mut unit_locked.specialized {
        srvc.status = ServiceStatus::Stopped;

        if let Some(conf) = &srvc.service_config {
            if conf.keep_alive {
                srvc.start(
                    srvc_id,
                    &unit_locked.conf.name(),
                    &sockets,
                    pid_table,
                    notification_socket_path,
                    &Vec::new(),
                    true,
                )?;
            } else {
                trace!(
                    "Killing all services requiring service with id {}: {:?}",
                    srvc_id,
                    unit_locked.install.required_by
                );
                let pid_table_locked = &mut *pid_table.lock().unwrap();
                let unit_table_locked = &*unit_table.read().unwrap();
                super::kill_service::kill_services(
                    unit_locked.install.required_by.clone(),
                    unit_table_locked,
                    pid_table_locked,
                );
            }
        }
    }
    Ok(())
}
