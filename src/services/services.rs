use super::start_service::*;
use crate::platform::EventFd;
use crate::units::*;
use std::error::Error;
use std::os::unix::io::RawFd;
use std::os::unix::net::UnixDatagram;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Debug)]
pub struct ServiceRuntimeInfo {
    pub restarted: u64,
    pub up_since: Option<std::time::Instant>,
}

#[derive(Debug)]
pub struct Service {
    pub pid: Option<nix::unistd::Pid>,
    pub service_config: Option<ServiceConfig>,

    pub socket_names: Vec<String>,

    pub status_msgs: Vec<String>,

    pub process_group: Option<nix::unistd::Pid>,

    pub runtime_info: ServiceRuntimeInfo,
    pub signaled_ready: bool,

    pub notifications: Option<Arc<Mutex<UnixDatagram>>>,
    pub notifications_path: Option<std::path::PathBuf>,

    pub stdout_dup: Option<(RawFd, RawFd)>,
    pub stderr_dup: Option<(RawFd, RawFd)>,
    pub notifications_buffer: String,
    pub stdout_buffer: Vec<u8>,
    pub stderr_buffer: Vec<u8>,
}

pub enum StartResult {
    Started,
    WaitingForSocket,
}

impl Service {
    pub fn start(
        &mut self,
        id: UnitId,
        name: &str,
        fd_store: ArcMutFDStore,
        pid_table: ArcMutPidTable,
        notification_socket_path: std::path::PathBuf,
        eventfds: &[EventFd],
        allow_ignore: bool,
    ) -> Result<StartResult, String> {
        if self.pid.is_some() {
            return Err(format!(
                "Service {} has already a pid {:?}",
                name,
                self.pid.unwrap()
            ));
        }
        if self.process_group.is_some() {
            return Err(format!(
                "Service {} has already a pgid {:?}",
                name,
                self.process_group.unwrap()
            ));
        }
        if !allow_ignore || self.socket_names.is_empty() {
            trace!("Start service {}", name);
            super::prepare_service::prepare_service(self, name, &notification_socket_path)?;
            self.run_prestart(id, name, pid_table.clone());
            {
                let mut pid_table_locked = pid_table.lock().unwrap();
                // This mainly just forks the process. The waiting (if necessary) is done below
                // Doing it under the lock of the pid_table prevents races between processes exiting very
                // fast and inserting the new pid into the pid table
                start_service(self, name.clone(), &*fd_store.read().unwrap())?;
                if let Some(new_pid) = self.pid {
                    pid_table_locked.insert(new_pid, PidEntry::Service(id));
                    crate::platform::notify_event_fds(&eventfds);
                }
            }
            if let Some(sock) = &self.notifications {
                let sock = sock.clone();
                super::fork_parent::wait_for_service(self, name, &*sock.lock().unwrap())?;
            }
            self.run_poststart(id, name, pid_table.clone());
            Ok(StartResult::Started)
        } else {
            trace!(
                "Ignore service {} start, waiting for socket activation instead",
                name,
            );
            crate::platform::notify_event_fds(&eventfds);
            Ok(StartResult::WaitingForSocket)
        }
    }

    fn stop(&mut self, id: UnitId, name: &str, pid_table: ArcMutPidTable) {
        self.run_stop_cmd(id, name, pid_table);

        if let Some(proc_group) = self.process_group {
            match nix::sys::signal::kill(proc_group, nix::sys::signal::Signal::SIGKILL) {
                Ok(_) => trace!("Success killing process group for service {}", name,),
                Err(e) => error!("Error killing process group for service {}: {}", name, e,),
            }
        } else {
            trace!("Tried to kill service that didn't have a process-group. This might have resulted in orphan processes.");
        }
        self.pid = None;
        self.process_group = None;
    }

    pub fn kill(&mut self, id: UnitId, name: &str, pid_table: ArcMutPidTable) {
        self.stop(id, name, pid_table);
    }

    fn run_stop_cmd(&self, id: UnitId, name: &str, pid_table: ArcMutPidTable) {
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

        trace!("Run stop process for service: {} with pid", name);
        // TODO alter this to use the stdout/err pipes after the fork
        // TODO check return value
        match cmd.spawn() {
            Ok(mut child) => {
                pid_table.lock().unwrap().insert(
                    nix::unistd::Pid::from_raw(child.id() as i32),
                    PidEntry::Stop(id),
                );
                trace!("Wait for stop process for service {}", name);
                match wait_for_child(&mut child, None) {
                    WaitResult::Success(Err(e)) => {
                        // This might also happen because it was collected by the signal_handler.
                        // This could be fixed by using the waitid() with WNOWAIT in the signal handler but
                        // that has not been ported to rust
                        error!(
                            "Error while waiting on stop process for service {}: {}",
                            name, e
                        );
                        // TODO return error or something
                    }
                    WaitResult::Success(Ok(_)) => {
                        // Happy
                    }
                    WaitResult::TimedOut => {
                        // TODO handle timeout
                        let _ = child.kill();
                    }
                }
                trace!(
                    "Ran stop process for service: {} with pid: {:?}",
                    name,
                    child.id()
                );
                pid_table
                    .lock()
                    .unwrap()
                    .remove(&nix::unistd::Pid::from_raw(child.id() as i32));
            }
            Err(e) => panic!(e.description().to_owned()),
        }
    }
    fn run_prestart(&self, id: UnitId, name: &str, pid_table: ArcMutPidTable) {
        let split: Vec<&str> = match &self.service_config {
            Some(conf) => {
                if conf.startpre.is_empty() {
                    return;
                }
                conf.startpre.split(' ').collect()
            }
            None => return,
        };
        let mut cmd = Command::new(split[0]);
        for part in &split[1..] {
            cmd.arg(part);
        }
        cmd.stdout(Stdio::null());
        trace!("Start running prestart for service: {}", name);
        // TODO alter this to use the stdout/err pipes after the fork
        // TODO check return value
        match cmd.spawn() {
            Ok(mut child) => {
                pid_table.lock().unwrap().insert(
                    nix::unistd::Pid::from_raw(child.id() as i32),
                    PidEntry::Stop(id),
                );
                trace!("Wait for prestart for service {}", name);
                match wait_for_child(&mut child, None) {
                    WaitResult::Success(Err(e)) => {
                        // This might also happen because it was collected by the signal_handler.
                        // This could be fixed by using the waitid() with WNOWAIT in the signal handler but
                        // that has not been ported to rust
                        error!(
                            "Error while waiting on prestart for service {}: {}",
                            name, e
                        );
                        // TODO return error or something
                    }
                    WaitResult::Success(Ok(_)) => {
                        // Happy
                    }
                    WaitResult::TimedOut => {
                        // TODO handle timeout
                        let _ = child.kill();
                    }
                }
                trace!(
                    "Ran prestart for service: {} with pid: {:?}",
                    name,
                    child.id()
                );
                pid_table
                    .lock()
                    .unwrap()
                    .remove(&nix::unistd::Pid::from_raw(child.id() as i32));
            }
            Err(e) => panic!(e.description().to_owned()),
        }
    }
    fn run_poststart(&self, id: UnitId, name: &str, pid_table: ArcMutPidTable) {
        let split: Vec<&str> = match &self.service_config {
            Some(conf) => {
                if conf.startpost.is_empty() {
                    return;
                }
                conf.startpost.split(' ').collect()
            }
            None => return,
        };
        let mut cmd = Command::new(split[0]);
        for part in &split[1..] {
            cmd.arg(part);
        }
        cmd.stdout(Stdio::null());
        trace!("Start running proststart for service: {}", name);
        // TODO alter this to use the stdout/err pipes after the fork
        match cmd.spawn() {
            Ok(mut child) => {
                pid_table.lock().unwrap().insert(
                    nix::unistd::Pid::from_raw(child.id() as i32),
                    PidEntry::Stop(id),
                );
                trace!("Wait for poststart for service {}", name);
                match wait_for_child(&mut child, None) {
                    WaitResult::Success(Err(e)) => {
                        // This might also happen because it was collected by the signal_handler.
                        // This could be fixed by using the waitid() with WNOWAIT in the signal handler but
                        // that has not been ported to rust
                        error!(
                            "Error while waiting on poststart for service {}: {}",
                            name, e
                        );
                        // TODO return error or something
                    }
                    WaitResult::Success(Ok(_)) => {
                        // Happy
                    }
                    WaitResult::TimedOut => {
                        // TODO handle timeout
                        let _ = child.kill();
                    }
                }
                trace!(
                    "Ran proststart for service: {} with pid: {:?}",
                    name,
                    child.id()
                );
                pid_table
                    .lock()
                    .unwrap()
                    .remove(&nix::unistd::Pid::from_raw(child.id() as i32));
            }
            Err(e) => panic!(e.description().to_owned()),
        }
    }
}

enum WaitResult {
    TimedOut,
    Success(std::io::Result<Option<std::process::ExitStatus>>),
}

/// Wait for the termination of a subprocess, with an optional timeout.
/// An error does not mean that the waiting actually failed.
/// This might also happen because it was collected by the signal_handler.
/// This could be fixed by using the waitid() with WNOWAIT in the signal handler but
/// that has not been ported to rust
fn wait_for_child(
    child: &mut std::process::Child,
    time_out: Option<std::time::Duration>,
) -> WaitResult {
    let mut counter = 1u64;
    let start_time = std::time::Instant::now();
    loop {
        if let Some(time_out) = time_out {
            if start_time.elapsed() >= time_out {
                return WaitResult::TimedOut;
            }
        }
        match child.try_wait() {
            Err(e) => {
                // This might also happen because it was collected by the signal_handler.
                // This could be fixed by using the waitid() with WNOWAIT in the signal handler but
                // that has not been ported to rust

                // in any case it probably means that the process has exited...
                return WaitResult::Success(Err(e));
            }
            Ok(Some(x)) => {
                // Happy
                return WaitResult::Success(Ok(Some(x)));
            }
            Ok(None) => {
                // Happy but need to wait longer
            }
        }
        // exponential backoff to get low latencies for fast processes
        // but not hog the cpu for too long
        // start at 0.05 ms
        // capped to 10 ms to not introduce too big latencies
        // TODO review those numbers
        let sleep_dur = std::time::Duration::from_micros(counter * 50);
        let sleep_cap = std::time::Duration::from_millis(10);
        let sleep_dur = sleep_dur.min(sleep_cap);
        if sleep_dur < sleep_cap {
            counter = counter * 2;
        }
        std::thread::sleep(sleep_dur);
    }
}