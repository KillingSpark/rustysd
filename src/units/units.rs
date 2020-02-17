use crate::fd_store::FDStore;
use crate::platform::EventFd;
use crate::services::Service;
use crate::sockets::{Socket, SocketKind, SpecializedSocketConfig};
use crate::units::*;

use nix::unistd::Pid;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::{fmt, path::PathBuf};

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum UnitIdKind {
    Target,
    Socket,
    Service,
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct UnitId(pub UnitIdKind, pub u64);

impl fmt::Debug for UnitId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(format!("{}", self.1).as_str())
    }
}

impl fmt::Display for UnitId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(format!("{:?}", self).as_str())
    }
}

impl std::cmp::PartialOrd for UnitId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.1.partial_cmp(&other.1)
    }
}

impl std::cmp::Ord for UnitId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.1.cmp(&other.1)
    }
}

pub type UnitTable = HashMap<UnitId, Arc<Mutex<Unit>>>;
pub type ArcMutUnitTable = Arc<RwLock<UnitTable>>;

pub type StatusTable = HashMap<UnitId, Arc<Mutex<UnitStatus>>>;
pub type ArcMutStatusTable = Arc<RwLock<StatusTable>>;

pub type PidTable = HashMap<Pid, PidEntry>;
pub type ArcMutPidTable = Arc<Mutex<PidTable>>;

pub type ArcMutFDStore = Arc<RwLock<FDStore>>;

pub struct RuntimeInfo {
    pub unit_table: ArcMutUnitTable,
    pub status_table: ArcMutStatusTable,
    pub pid_table: ArcMutPidTable,
    pub fd_store: ArcMutFDStore,
    pub config: crate::config::Config,
    pub last_id: Arc<Mutex<u64>>,
}

// This will be passed through to all the different threads as a central state struct
pub type ArcRuntimeInfo = Arc<RuntimeInfo>;

pub fn lock_all(
    units: &mut Vec<(UnitId, Arc<Mutex<Unit>>)>,
) -> HashMap<UnitId, std::sync::MutexGuard<'_, Unit>> {
    let mut units_locked = HashMap::new();
    // sort to make sure units always get locked in the same ordering
    units.sort_by(|(lid, _), (rid, _)| lid.cmp(rid));

    for (id, unit) in units {
        trace!("Lock unit: {:?}", id);
        let other_unit_locked = unit.lock().unwrap();
        trace!("Locked unit: {:?}", id);
        units_locked.insert(id.clone(), other_unit_locked);
    }

    units_locked
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum PidEntry {
    Service(UnitId, ServiceType),
    OneshotExited(crate::signal_handler::ChildTermination),
    Helper(UnitId, String),
    HelperExited(crate::signal_handler::ChildTermination),
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum UnitStatus {
    NeverStarted,
    Starting,
    Started,
    StartedWaitingForSocket,
    Stopping,
    Stopped,
    StoppedFinal(String),
}

#[derive(Debug)]
pub enum UnitSpecialized {
    Socket(Socket),
    Service(Service),
    Target,
}

#[derive(Debug, Default)]
/// These vecs are meant like this:
/// Install::after: this unit should start after these units have been started
/// Install::before: this unit should start before these units have been started
/// ....
pub struct Install {
    pub wants: Vec<UnitId>,
    pub requires: Vec<UnitId>,

    pub wanted_by: Vec<UnitId>,
    pub required_by: Vec<UnitId>,

    pub before: Vec<UnitId>,
    pub after: Vec<UnitId>,

    pub install_config: Option<InstallConfig>,
}

pub struct Unit {
    pub id: UnitId,
    pub conf: UnitConfig,
    pub specialized: UnitSpecialized,

    pub install: Install,
}

impl Unit {
    pub fn is_service(&self) -> bool {
        if let UnitSpecialized::Service(_) = self.specialized {
            true
        } else {
            false
        }
    }
    pub fn is_socket(&self) -> bool {
        if let UnitSpecialized::Socket(_) = self.specialized {
            true
        } else {
            false
        }
    }
    pub fn is_target(&self) -> bool {
        if let UnitSpecialized::Target = self.specialized {
            true
        } else {
            false
        }
    }

    pub fn dedup_dependencies(&mut self) {
        self.install.wants.sort();
        self.install.wanted_by.sort();
        self.install.required_by.sort();
        self.install.before.sort();
        self.install.after.sort();
        self.install.requires.sort();
        // dedup after sorting
        self.install.wants.dedup();
        self.install.requires.dedup();
        self.install.wanted_by.dedup();
        self.install.required_by.dedup();
        self.install.before.dedup();
        self.install.after.dedup();
    }

    pub fn activate(
        &mut self,
        run_info: ArcRuntimeInfo,
        notification_socket_path: std::path::PathBuf,
        eventfds: &[EventFd],
        allow_ignore: bool,
    ) -> Result<UnitStatus, UnitOperationError> {
        match &mut self.specialized {
            UnitSpecialized::Target => trace!("Reached target {}", self.conf.name()),
            UnitSpecialized::Socket(sock) => {
                sock.open_all(
                    self.conf.name(),
                    self.id,
                    &mut *run_info.fd_store.write().unwrap(),
                )
                .map_err(|e| UnitOperationError {
                    unit_name: self.conf.name(),
                    unit_id: self.id,
                    reason: UnitOperationErrorReason::SocketOpenError(format!("{}", e)),
                })?;
            }
            UnitSpecialized::Service(srvc) => {
                match srvc
                    .start(
                        self.id,
                        &self.conf.name(),
                        run_info,
                        notification_socket_path,
                        eventfds,
                        allow_ignore,
                    )
                    .map_err(|e| UnitOperationError {
                        unit_name: self.conf.name(),
                        unit_id: self.id,
                        reason: UnitOperationErrorReason::ServiceStartError(e),
                    })? {
                    crate::services::StartResult::Started => return Ok(UnitStatus::Started),
                    crate::services::StartResult::WaitingForSocket => {
                        return Ok(UnitStatus::StartedWaitingForSocket)
                    }
                }
            }
        }
        Ok(UnitStatus::Started)
    }
    pub fn deactivate(&mut self, run_info: ArcRuntimeInfo) -> Result<(), UnitOperationError> {
        trace!("Deactivate unit: {}", self.conf.name());
        match &mut self.specialized {
            UnitSpecialized::Target => { /* nothing to do */ }
            UnitSpecialized::Socket(sock) => {
                sock.close_all(self.conf.name(), &mut *run_info.fd_store.write().unwrap())
                    .map_err(|e| UnitOperationError {
                        unit_name: self.conf.name(),
                        unit_id: self.id,
                        reason: UnitOperationErrorReason::SocketCloseError(e),
                    })?;
            }
            UnitSpecialized::Service(srvc) => {
                srvc.kill(self.id, &self.conf.name(), run_info)
                    .map_err(|e| UnitOperationError {
                        unit_name: self.conf.name(),
                        unit_id: self.id,
                        reason: UnitOperationErrorReason::ServiceStopError(e),
                    })?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct UnitConfig {
    pub filepath: PathBuf,

    pub description: String,

    pub wants: Vec<String>,
    pub requires: Vec<String>,
    pub before: Vec<String>,
    pub after: Vec<String>,
}

impl UnitConfig {
    pub fn name(&self) -> String {
        let name = self
            .filepath
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();

        //let split: Vec<_> = name.split('.').collect();
        //split[0..split.len() - 1].join(".")
        name
    }
    pub fn name_without_suffix(&self) -> String {
        let name = self.name();
        let split: Vec<_> = name.split('.').collect();
        split[0..split.len() - 1].join(".")
    }
}

#[derive(Clone)]
pub struct SocketConfig {
    pub kind: SocketKind,
    pub specialized: SpecializedSocketConfig,
}

impl fmt::Debug for SocketConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "SocketConfig {{ kind: {:?}, specialized: {:?} }}",
            self.kind, self.specialized
        )?;
        Ok(())
    }
}

unsafe impl Send for SocketConfig {}

#[derive(Debug)]
pub struct InstallConfig {
    pub wanted_by: Vec<String>,
    pub required_by: Vec<String>,
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum ServiceType {
    Simple,
    Notify,
    Dbus,
    OneShot,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum NotifyKind {
    Main,
    Exec,
    All,
    None,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum ServiceRestart {
    Always,
    No,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Timeout {
    Duration(std::time::Duration),
    Infinity,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ExecConfig {
    pub user: Option<String>,
    pub group: Option<String>,
    pub supplementary_groups: Vec<String>,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum CommandlinePrefix {
    AtSign,
    Minus,
    Colon,
    Plus,
    Exclamation,
    DoubleExclamation,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Commandline {
    pub cmd: String,
    pub args: Vec<String>,
    pub prefixes: Vec<CommandlinePrefix>,
}

impl ToString for Commandline {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ServiceConfig {
    pub restart: ServiceRestart,
    pub accept: bool,
    pub notifyaccess: NotifyKind,
    pub exec: Commandline,
    pub stop: Vec<Commandline>,
    pub stoppost: Vec<Commandline>,
    pub startpre: Vec<Commandline>,
    pub startpost: Vec<Commandline>,
    pub srcv_type: ServiceType,
    pub starttimeout: Option<Timeout>,
    pub stoptimeout: Option<Timeout>,
    pub generaltimeout: Option<Timeout>,

    pub exec_config: ExecConfig,

    pub dbus_name: Option<String>,

    pub sockets: Vec<String>,
}
