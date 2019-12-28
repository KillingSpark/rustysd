use super::start_service::*;
use std::collections::HashMap;
use std::error::Error;
use std::os::unix::io::RawFd;
use std::os::unix::net::UnixDatagram;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::Mutex;

use crate::platform::EventFd;
use crate::sockets::Socket;
use crate::units::*;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ServiceStatus {
    NeverRan,
    Starting,
    Running,
    Stopped,
    StoppedFinal,
}

impl ToString for ServiceStatus {
    fn to_string(&self) -> String {
        match *self {
            ServiceStatus::NeverRan => "NeverRan".into(),
            ServiceStatus::Running => "Running".into(),
            ServiceStatus::Starting => "Starting".into(),
            ServiceStatus::Stopped => "Stopped".into(),
            ServiceStatus::StoppedFinal => "StoppedFinal".into(),
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
        sockets: &mut HashMap<InternalId, &mut Socket>,
        pids: ArcMutPidTable,
        notification_socket_path: std::path::PathBuf,
        eventfds: &[EventFd],
        allow_ignore: bool,
    ) -> Result<(), String> {
        trace!("Start service {}", name);

        match self.status {
            ServiceStatus::NeverRan | ServiceStatus::Stopped => {
                if !allow_ignore || self.socket_ids.is_empty() {
                    start_service(self, name.clone(), &sockets, notification_socket_path)?;

                    if let Some(new_pid) = self.pid {
                        {
                            let mut pids = pids.lock().unwrap();
                            pids.insert(new_pid, PidEntry::Service(id));
                        }
                        crate::platform::notify_event_fds(&eventfds)
                    }
                } else {
                    trace!(
                        "Ignore service {} start, waiting for socket activation instead",
                        name,
                    );
                    for sock in sockets.values_mut() {
                        sock.activated = false;
                    }
                    crate::platform::notify_event_fds(&eventfds)
                }
            }
            _ => error!(
                "Tried to start service {} after it was already running",
                name
            ),
        }
        Ok(())
    }

    fn stop(&mut self, id: InternalId, name: &str, pid_table: &mut PidTable) {
        self.run_stop_cmd(id, name, pid_table);

        if let Some(proc_group) = self.process_group {
            match nix::sys::signal::kill(proc_group, nix::sys::signal::Signal::SIGKILL) {
                Ok(_) => trace!("Success killing process group for service {}", name,),
                Err(e) => error!("Error killing process group for service {}: {}", name, e,),
            }
        }
    }

    pub fn kill(&mut self, id: InternalId, name: &str, pid_table: &mut PidTable) {
        self.status = ServiceStatus::Stopped;
        self.stop(id, name, pid_table);
    }
    pub fn kill_final(&mut self, id: InternalId, name: &str, pid_table: &mut PidTable) {
        self.status = ServiceStatus::StoppedFinal;
        self.stop(id, name, pid_table);
    }

    pub fn run_stop_cmd(&self, id: InternalId, name: &str, pid_table: &mut PidTable) {
        let split: Vec<&str> = match &self.service_config {
            Some(conf) => {
                if conf.stop.is_empty() {
                    return;
                }
                conf.stop.split(' ').collect()
            }
            None => return,
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
                    PidEntry::Stop(id),
                );
                trace!("Stopped Service: {} with pid: {:?}", name, self.pid);
            }
            Err(e) => panic!(e.description().to_owned()),
        }
    }
}

pub fn service_exit_handler(
    pid: nix::unistd::Pid,
    code: i32,
    unit_table: ArcMutUnitTable,
    pid_table: ArcMutPidTable,
    notification_socket_path: std::path::PathBuf,
    eventfds: &[EventFd],
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

    let unit = {
        let units_locked = unit_table.read().unwrap();
        match units_locked.get(&srvc_id) {
            Some(unit) => Arc::clone(unit),
            None => {
                panic!("Tried to run a unit that has been removed from the map");
            }
        }
    };
    let name = unit.lock().unwrap().conf.name();
    {
        let restart_unit = {
            let unit_locked = &mut *unit.lock().unwrap();
            if let UnitSpecialized::Service(srvc) = &mut unit_locked.specialized {
                trace!(
                    "Service with id: {}, name: {} pid: {} exited with code: {}",
                    srvc_id,
                    unit_locked.conf.name(),
                    pid,
                    code
                );
                if srvc.status == ServiceStatus::StoppedFinal {
                    false
                } else {
                    srvc.status = ServiceStatus::Stopped;
                    if let Some(conf) = &srvc.service_config {
                        if conf.restart == ServiceRestart::Always {
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
            } else {
                false
            }
        };
        if restart_unit {
            trace!("Restart service {} after it died", name);
            crate::units::activate_unit(
                srvc_id,
                None,
                unit_table,
                pid_table,
                notification_socket_path,
                Arc::new(eventfds.to_vec()),
                true,
            )?;
        } else {
            let unit_locked = unit.lock().unwrap();
            trace!(
                "Killing all services requiring service {}: {:?}",
                name,
                unit_locked.install.required_by
            );
            super::kill_service::kill_services(
                unit_locked.install.required_by.clone(),
                unit_table,
                pid_table,
            );
        }
    }
    Ok(())
}
