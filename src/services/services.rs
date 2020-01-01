use super::start_service::*;
use std::collections::HashMap;
use std::error::Error;
use std::os::unix::io::RawFd;
use std::os::unix::net::UnixDatagram;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::Mutex;

use crate::platform::EventFd;
use crate::signal_handler::ChildTermination;
use crate::sockets::Socket;
use crate::units::*;

#[derive(Debug)]
pub struct ServiceRuntimeInfo {
    pub restarted: u64,
    pub up_since: Option<std::time::Instant>,
}

#[derive(Debug)]
pub struct Service {
    pub pid: Option<nix::unistd::Pid>,
    pub service_config: Option<ServiceConfig>,

    pub socket_ids: Vec<UnitId>,

    pub status_msgs: Vec<String>,

    pub process_group: Option<nix::unistd::Pid>,

    pub runtime_info: ServiceRuntimeInfo,
    pub signaled_ready: bool,

    pub notifications: Option<Arc<Mutex<UnixDatagram>>>,
    pub stdout_dup: Option<(RawFd, RawFd)>,
    pub stderr_dup: Option<(RawFd, RawFd)>,
    pub notifications_buffer: String,
    pub stdout_buffer: Vec<u8>,
    pub stderr_buffer: Vec<u8>,
}

impl Service {
    pub fn start(
        &mut self,
        id: UnitId,
        name: &String,
        sockets: &mut HashMap<UnitId, &mut Socket>,
        pid_table: ArcMutPidTable,
        notification_socket_path: std::path::PathBuf,
        eventfds: &[EventFd],
        allow_ignore: bool,
    ) -> Result<(), String> {
        trace!("Start service {}", name);
        if !allow_ignore || self.socket_ids.is_empty() {
            let mut pid_table = pid_table.lock().unwrap();
            start_service(self, name.clone(), &sockets, notification_socket_path)?;
            if let Some(new_pid) = self.pid {
                pid_table.insert(new_pid, PidEntry::Service(id));
                for sock in sockets.values_mut() {
                    sock.activated = true;
                }
                crate::platform::notify_event_fds(&eventfds);
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
        Ok(())
    }

    fn stop(&mut self, id: UnitId, name: &str, pid_table: &mut PidTable) {
        self.run_stop_cmd(id, name, pid_table);

        if let Some(proc_group) = self.process_group {
            match nix::sys::signal::kill(proc_group, nix::sys::signal::Signal::SIGKILL) {
                Ok(_) => trace!("Success killing process group for service {}", name,),
                Err(e) => error!("Error killing process group for service {}: {}", name, e,),
            }
        }
        self.pid = None;
        self.process_group = None;
    }

    pub fn kill(&mut self, id: UnitId, name: &str, pid_table: &mut PidTable) {
        self.stop(id, name, pid_table);
    }

    pub fn run_stop_cmd(&self, id: UnitId, name: &str, pid_table: &mut PidTable) {
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
    code: ChildTermination,
    run_info: ArcRuntimeInfo,
    notification_socket_path: std::path::PathBuf,
    eventfds: &[EventFd],
) -> Result<(), String> {
    trace!("Exit handler with pid: {}", pid);
    let srvc_id = {
        let unit_table_locked = run_info.unit_table.read().unwrap();
        let entry = {
            let pid_table_locked = &mut *run_info.pid_table.lock().unwrap();
            pid_table_locked.get(&pid).map(|x| {
                let y: PidEntry = *x;
                y
            })
        };
        match entry {
            Some(entry) => match entry {
                PidEntry::Service(id) => id,
                PidEntry::Stop(id) => {
                    trace!(
                        "Stop process for service: {} exited with: {:?}",
                        unit_table_locked
                            .get(&id)
                            .unwrap()
                            .lock()
                            .unwrap()
                            .conf
                            .name(),
                        code
                    );
                    let pid_table_locked = &mut *run_info.pid_table.lock().unwrap();
                    pid_table_locked.remove(&pid);
                    return Ok(());
                }
            },
            None => {
                warn!("All spawned processes should have a pid entry");
                return Ok(());
            }
        }
    };

    let unit = {
        let unit_table_locked = run_info.unit_table.read().unwrap();
        match unit_table_locked.get(&srvc_id) {
            Some(unit) => Arc::clone(unit),
            None => {
                panic!("Tried to run a unit that has been removed from the map");
            }
        }
    };

    let (name, restart_unit) = {
        let unit_locked = &mut *unit.lock().unwrap();
        let name = unit_locked.conf.name();
        if let UnitSpecialized::Service(srvc) = &mut unit_locked.specialized {
            trace!(
                "Service with id: {:?}, name: {} pid: {} exited with: {:?}",
                srvc_id,
                unit_locked.conf.name(),
                pid,
                code
            );

            if let Some(conf) = &srvc.service_config {
                if conf.restart == ServiceRestart::Always {
                    (name, true)
                } else {
                    (name, false)
                }
            } else {
                (name, false)
            }
        } else {
            (name, false)
        }
    };
    if restart_unit {
        trace!("Restart service {} after it died", name);
        crate::units::reactivate_unit(
            srvc_id,
            run_info,
            notification_socket_path,
            Arc::new(eventfds.to_vec()),
        )?;
    } else {
        let unit_locked = unit.lock().unwrap();
        trace!(
            "Killing all services requiring service {}: {:?}",
            name,
            unit_locked.install.required_by
        );
        crate::units::deactivate_units(unit_locked.install.required_by.clone(), run_info.clone());
    }
    Ok(())
}
