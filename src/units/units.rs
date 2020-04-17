use crate::platform::EventFd;
use crate::services::Service;
use crate::sockets::{Socket, SocketKind, SpecializedSocketConfig};
use crate::units::*;

use std::fmt;
use std::sync::RwLock;

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum UnitIdKind {
    Target,
    Socket,
    Service,
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct UnitId {
    pub kind: UnitIdKind,
    pub name: String,
}
impl UnitId {
    pub fn name_without_suffix(&self) -> String {
        let split: Vec<_> = self.name.split('.').collect();
        split[0..split.len() - 1].join(".")
    }
}

impl fmt::Debug for UnitId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(format!("{}", self.name).as_str())
    }
}

impl fmt::Display for UnitId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(format!("{:?}", self).as_str())
    }
}

impl std::cmp::PartialOrd for UnitId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl std::cmp::Ord for UnitId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialEq<str> for UnitId {
    fn eq(&self, other: &str) -> bool {
        self.name.eq(other)
    }
}
impl PartialEq<String> for UnitId {
    fn eq(&self, other: &String) -> bool {
        self.name.eq(other)
    }
}
impl PartialEq<dyn AsRef<str>> for UnitId {
    fn eq(&self, other: &dyn AsRef<str>) -> bool {
        self.name.eq(other.as_ref())
    }
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

impl UnitStatus {
    pub fn is_stopped(&self) -> bool {
        match self {
            UnitStatus::StoppedFinal(_) => true,
            UnitStatus::Stopped => true,
            _ => false,
        }
    }
}

pub struct Unit {
    pub id: UnitId,
    pub common: Common,
    pub specific: Specific,
}

pub struct Common {
    pub unit: UnitConfig,
    pub dependencies: Dependencies,
    pub status: RwLock<UnitStatus>,
}

pub enum Specific {
    Service(ServiceSpecific),
    Socket(SocketSpecific),
    Target(TargetSpecific),
}

pub struct ServiceSpecific {
    pub conf: ServiceConfig,
    pub state: RwLock<ServiceState>,
}

impl ServiceSpecific {
    pub fn has_socket(&self, socket: &str) -> bool {
        self.conf.sockets.iter().any(|id| id.eq(socket))
    }
}

pub struct SocketSpecific {
    pub conf: SocketConfig,
    pub state: RwLock<SocketState>,
}
pub struct TargetSpecific {}

#[derive(Default)]
pub struct CommonState {
    pub up_since: Option<std::time::Instant>,
    pub restart_count: u64,
}
pub struct ServiceState {
    pub common: CommonState,
    pub srvc: Service,
}
pub struct SocketState {
    pub common: CommonState,
    pub sock: Socket,
}
pub struct TargetState {
    pub common: CommonState,
}

impl Unit {
    pub fn is_service(&self) -> bool {
        if let UnitIdKind::Service = self.id.kind {
            true
        } else {
            false
        }
    }
    pub fn is_socket(&self) -> bool {
        if let UnitIdKind::Socket = self.id.kind {
            true
        } else {
            false
        }
    }
    pub fn is_target(&self) -> bool {
        if let UnitIdKind::Target = self.id.kind {
            true
        } else {
            false
        }
    }

    pub fn name_without_suffix(&self) -> String {
        let split: Vec<_> = self.id.name.split('.').collect();
        split[0..split.len() - 1].join(".")
    }

    pub fn dedup_dependencies(&mut self) {
        self.common.dependencies.dedup();
    }

    pub fn activate(
        &self,
        run_info: &RuntimeInfo,
        notification_socket_path: std::path::PathBuf,
        eventfds: &[EventFd],
        allow_ignore: bool,
    ) -> Result<UnitStatus, UnitOperationError> {
        // TODO change status here!
        match &self.specific {
            Specific::Target(_) => trace!("Reached target {}", self.id.name),
            Specific::Socket(specific) => {
                let state = &mut *specific.state.write().unwrap();
                state
                    .sock
                    .open_all(
                        &specific.conf,
                        self.id.name.clone(),
                        self.id.clone(),
                        &mut *run_info.fd_store.write().unwrap(),
                    )
                    .map_err(|e| UnitOperationError {
                        unit_name: self.id.name.clone(),
                        unit_id: self.id.clone(),
                        reason: UnitOperationErrorReason::SocketOpenError(format!("{}", e)),
                    })?;
            }
            Specific::Service(specific) => {
                let state = &mut *specific.state.write().unwrap();
                match state
                    .srvc
                    .start(
                        &specific.conf,
                        self.id.clone(),
                        &self.id.name,
                        run_info,
                        notification_socket_path,
                        eventfds,
                        allow_ignore,
                    )
                    .map_err(|e| UnitOperationError {
                        unit_name: self.id.name.clone(),
                        unit_id: self.id.clone(),
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
    pub fn deactivate(&self, run_info: &RuntimeInfo) -> Result<(), UnitOperationError> {
        // TODO change status here!
        trace!("Deactivate unit: {}", self.id.name);
        match &self.specific {
            Specific::Target(_) => { /* nothing to do */ }
            Specific::Socket(specific) => {
                let state = &mut *specific.state.write().unwrap();
                state
                    .sock
                    .close_all(
                        &specific.conf,
                        self.id.name.clone(),
                        &mut *run_info.fd_store.write().unwrap(),
                    )
                    .map_err(|e| UnitOperationError {
                        unit_name: self.id.name.clone(),
                        unit_id: self.id.clone(),
                        reason: UnitOperationErrorReason::SocketCloseError(e),
                    })?;
            }
            Specific::Service(specific) => {
                let state = &mut *specific.state.write().unwrap();
                state
                    .srvc
                    .kill(&specific.conf, self.id.clone(), &self.id.name, run_info)
                    .map_err(|e| UnitOperationError {
                        unit_name: self.id.name.clone(),
                        unit_id: self.id.clone(),
                        reason: UnitOperationErrorReason::ServiceStopError(e),
                    })?;
            }
        }
        Ok(())
    }
}

pub fn collect_names_needed(new_unit: &units::Unit, names_needed: &mut Vec<String>) {
    names_needed.extend(
        new_unit
            .common
            .dependencies
            .after
            .iter()
            .cloned()
            .map(|id| id.name),
    );
    names_needed.extend(
        new_unit
            .common
            .dependencies
            .before
            .iter()
            .cloned()
            .map(|id| id.name),
    );
    names_needed.extend(
        new_unit
            .common
            .dependencies
            .wanted_by
            .iter()
            .cloned()
            .map(|id| id.name),
    );
    names_needed.extend(
        new_unit
            .common
            .dependencies
            .wants
            .iter()
            .cloned()
            .map(|id| id.name),
    );
    names_needed.extend(
        new_unit
            .common
            .dependencies
            .required_by
            .iter()
            .cloned()
            .map(|id| id.name),
    );
    names_needed.extend(
        new_unit
            .common
            .dependencies
            .requires
            .iter()
            .cloned()
            .map(|id| id.name),
    );

    if let Specific::Socket(sock) = &new_unit.specific {
        names_needed.extend(sock.conf.services.iter().cloned());
    }
    if let Specific::Service(srvc) = &new_unit.specific {
        names_needed.extend(srvc.conf.sockets.iter().cloned());
    }
}

#[derive(Debug, Clone)]
pub struct UnitConfig {
    pub description: String,
}

#[derive(Debug, Clone)]
/// These vecs are meant like this:
/// Install::after: this unit should start after these units have been started
/// Install::before: this unit should start before these units have been started
/// ....
pub struct Dependencies {
    pub wants: Vec<UnitId>,
    pub wanted_by: Vec<UnitId>,
    pub requires: Vec<UnitId>,
    pub required_by: Vec<UnitId>,
    pub before: Vec<UnitId>,
    pub after: Vec<UnitId>,
}

impl Dependencies {
    pub fn dedup(&mut self) {
        self.wants.sort();
        self.wanted_by.sort();
        self.required_by.sort();
        self.before.sort();
        self.after.sort();
        self.requires.sort();
        // dedup after sorting
        self.wants.dedup();
        self.requires.dedup();
        self.wanted_by.dedup();
        self.required_by.dedup();
        self.before.dedup();
        self.after.dedup();
    }

    /// Remove all occurences of this id from the vec
    fn remove_from_vec(ids: &mut Vec<UnitId>, id: &UnitId) {
        while let Some(idx) = ids.iter().position(|e| *e == *id) {
            ids.remove(idx);
        }
    }

    pub fn remove_id(&mut self, id: &UnitId) {
        Self::remove_from_vec(&mut self.wants, id);
        Self::remove_from_vec(&mut self.wanted_by, id);
        Self::remove_from_vec(&mut self.requires, id);
        Self::remove_from_vec(&mut self.required_by, id);
        Self::remove_from_vec(&mut self.before, id);
        Self::remove_from_vec(&mut self.after, id);
    }

    pub fn comes_after(&self, name: &str) -> bool {
        for id in &self.after {
            if id.eq(name) {
                return true;
            }
        }
        false
    }
    pub fn comes_before(&self, name: &str) -> bool {
        for id in &self.before {
            if id.eq(name) {
                return true;
            }
        }
        false
    }
    pub fn requires(&self, name: &str) -> bool {
        for id in &self.requires {
            if id.eq(name) {
                return true;
            }
        }
        false
    }
    pub fn required_by(&self, name: &str) -> bool {
        for id in &self.required_by {
            if id.eq(name) {
                return true;
            }
        }
        false
    }
    pub fn wants(&self, name: &str) -> bool {
        for id in &self.wants {
            if id.eq(name) {
                return true;
            }
        }
        false
    }
    pub fn wanted_by(&self, name: &str) -> bool {
        for id in &self.wanted_by {
            if id.eq(name) {
                return true;
            }
        }
        false
    }
}

#[derive(Clone, Debug)]
pub struct SingleSocketConfig {
    pub kind: SocketKind,
    pub specialized: SpecializedSocketConfig,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ExecConfig {
    pub user: nix::unistd::Uid,
    pub group: nix::unistd::Gid,
    pub supplementary_groups: Vec<nix::unistd::Gid>,
    pub stdout_path: Option<StdIoOption>,
    pub stderr_path: Option<StdIoOption>,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct PlatformSpecificServiceFields {
    pub cgroup_path: std::path::PathBuf,
}

#[cfg(not(target_os = "linux"))]
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct PlatformSpecificServiceFields {}

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

    pub platform_specific: PlatformSpecificServiceFields,

    pub dbus_name: Option<String>,

    pub sockets: Vec<String>,
}

pub struct SocketConfig {
    pub sockets: Vec<SingleSocketConfig>,
    pub filedesc_name: String,
    pub services: Vec<String>,

    pub exec_config: ExecConfig,
}
