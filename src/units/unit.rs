use crate::runtime_info::*;
use crate::services::Service;
use crate::sockets::{Socket, SocketKind, SpecializedSocketConfig};
use crate::units::*;

use std::sync::RwLock;

/// A units has a common part that all units share, like dependencies and a description. The specific part containbs mutable state and
/// the unit-type specific configs
pub struct Unit {
    pub id: UnitId,
    pub common: Common,
    pub specific: Specific,
}

/// Common attributes of units
pub struct Common {
    pub unit: UnitConfig,
    pub dependencies: Dependencies,
    pub status: RwLock<UnitStatus>,
}

/// Different unit-types have different configs and state
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

impl SocketSpecific {
    pub fn belongs_to_service(&self, service: &str) -> bool {
        self.conf.services.iter().any(|id| id.eq(service))
    }
}

pub struct TargetSpecific {}

#[derive(Default)]
/// All units have some common mutable state
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

    /// This activates the unit and manages the state transitions. It reports back the new unit status or any
    /// errors encountered while starting the unit. Note that these errors are also recorded in the units status.
    ///
    /// This always happens in this order:
    /// 1. Lock unit mutable state
    /// 1. Update status to 'Starting'
    /// 1. Do unit specific activation
    /// 1. Update status depending on the results
    /// 1. return
    pub fn activate(
        &self,
        run_info: &RuntimeInfo,
        source: ActivationSource,
    ) -> Result<UnitStatus, UnitOperationError> {
        match &self.specific {
            Specific::Target(_) => {
                {
                    let mut status = self.common.status.write().unwrap();
                    if status.is_started() {
                        return Ok(status.clone());
                    }
                    *status = UnitStatus::Started(StatusStarted::Running);
                }
                trace!("Reached target {}", self.id.name);
                Ok(UnitStatus::Started(StatusStarted::Running))
            }
            Specific::Socket(specific) => {
                let state = &mut *specific.state.write().unwrap();
                {
                    let mut status = self.common.status.write().unwrap();
                    if status.is_started() {
                        return Ok(status.clone());
                    }
                    *status = UnitStatus::Starting;
                }

                let open_res = state
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
                    });
                match open_res {
                    Ok(_) => {
                        let mut status = self.common.status.write().unwrap();
                        *status = UnitStatus::Started(StatusStarted::Running);
                        run_info.notify_eventfds();
                        Ok(UnitStatus::Started(StatusStarted::Running))
                    }
                    Err(e) => {
                        let mut status = self.common.status.write().unwrap();
                        *status = UnitStatus::Stopped(
                            StatusStopped::StoppedUnexpected,
                            vec![e.reason.clone()],
                        );
                        Err(e)
                    }
                }
            }
            Specific::Service(specific) => {
                let state = &mut *specific.state.write().unwrap();
                {
                    let mut status = self.common.status.write().unwrap();
                    match *status {
                        UnitStatus::Started(StatusStarted::WaitingForSocket) => {
                            if source.is_socket_activation() {
                                /* need activation */
                            } else {
                                return Ok(status.clone());
                            }
                        }
                        UnitStatus::Started(StatusStarted::Running) => {
                            return Ok(status.clone());
                        }
                        _ => { /* need activation */ }
                    }
                    *status = UnitStatus::Starting;
                }
                let start_res = state
                    .srvc
                    .start(
                        &specific.conf,
                        self.id.clone(),
                        &self.id.name,
                        run_info,
                        source,
                    )
                    .map_err(|e| UnitOperationError {
                        unit_name: self.id.name.clone(),
                        unit_id: self.id.clone(),
                        reason: UnitOperationErrorReason::ServiceStartError(e),
                    });
                match start_res {
                    Ok(crate::services::StartResult::Started) => {
                        {
                            let mut status = self.common.status.write().unwrap();
                            *status = UnitStatus::Started(StatusStarted::Running);
                        }
                        Ok(UnitStatus::Started(StatusStarted::Running))
                    }
                    Ok(crate::services::StartResult::WaitingForSocket) => {
                        {
                            let mut status = self.common.status.write().unwrap();
                            *status = UnitStatus::Started(StatusStarted::WaitingForSocket);
                        }
                        // tell socket activation to listen to these sockets again
                        for socket_id in &specific.conf.sockets {
                            if let Some(unit) = run_info.unit_table.get(socket_id) {
                                if let Specific::Socket(sock) = &unit.specific {
                                    let mut_state = &mut *sock.state.write().unwrap();
                                    mut_state.sock.activated = false;
                                }
                            }
                        }
                        run_info.notify_eventfds();
                        Ok(UnitStatus::Started(StatusStarted::WaitingForSocket))
                    }
                    Err(e) => {
                        let mut status = self.common.status.write().unwrap();
                        *status = UnitStatus::Stopped(
                            StatusStopped::StoppedUnexpected,
                            vec![e.reason.clone()],
                        );
                        Err(e)
                    }
                }
            }
        }
    }

    /// This dectivates the unit and manages the state transitions. It reports back any
    /// errors encountered while stopping the unit.
    ///
    /// This always happens in this order:
    /// 1. Lock unit mutable state
    /// 1. Update status to 'Stopping'
    /// 1. Do unit specific deactivation
    /// 1. Update status depending on the results
    /// 1. return
    pub fn deactivate(&self, run_info: &RuntimeInfo) -> Result<(), UnitOperationError> {
        // TODO change status here!
        trace!("Deactivate unit: {}", self.id.name);
        match &self.specific {
            Specific::Target(_) => {
                let mut status = self.common.status.write().unwrap();
                if status.is_stopped() {
                    return Ok(());
                }
                *status = UnitStatus::Stopped(StatusStopped::StoppedFinal, vec![]);
            }
            Specific::Socket(specific) => {
                let state = &mut *specific.state.write().unwrap();
                {
                    let mut status = self.common.status.write().unwrap();
                    if status.is_stopped() {
                        return Ok(());
                    }
                    *status = UnitStatus::Stopping;
                }
                let close_result = state
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
                    });
                match close_result {
                    Ok(_) => {
                        let mut status = self.common.status.write().unwrap();
                        *status = UnitStatus::Stopped(StatusStopped::StoppedFinal, vec![]);
                    }
                    Err(e) => {
                        let mut status = self.common.status.write().unwrap();
                        *status = UnitStatus::Stopped(
                            StatusStopped::StoppedFinal,
                            vec![e.reason.clone()],
                        );
                    }
                }
            }
            Specific::Service(specific) => {
                let state = &mut *specific.state.write().unwrap();
                {
                    let mut status = self.common.status.write().unwrap();
                    if status.is_stopped() {
                        return Ok(());
                    }
                    *status = UnitStatus::Stopping;
                }
                let kill_result = state
                    .srvc
                    .kill(&specific.conf, self.id.clone(), &self.id.name, run_info)
                    .map_err(|e| UnitOperationError {
                        unit_name: self.id.name.clone(),
                        unit_id: self.id.clone(),
                        reason: UnitOperationErrorReason::ServiceStopError(e),
                    });
                match kill_result {
                    Ok(_) => {
                        let mut status = self.common.status.write().unwrap();
                        *status = UnitStatus::Stopped(StatusStopped::StoppedFinal, vec![]);
                    }
                    Err(e) => {
                        let mut status = self.common.status.write().unwrap();
                        *status = UnitStatus::Stopped(
                            StatusStopped::StoppedFinal,
                            vec![e.reason.clone()],
                        );
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct UnitConfig {
    pub description: String,

    /// This is needed for adding/removing units. All units in this set must be present
    /// or this unit is considered invalid os it has to be removed too / cannot be added.
    pub refs_by_name: Vec<UnitId>,
}

#[derive(Debug, Clone)]
/// This are the runtime dependencies. They are extended when the unit is added into the unit set
/// so all dependencies go both ways.
///
/// These vecs are meant like this:
/// Dependencies::after: this unit should start after these units have been started
/// Dependencies::before: this unit should start before these units have been started
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

    pub fn kill_before_this(&self) -> Vec<UnitId> {
        let mut ids = Vec::new();
        ids.extend(self.required_by.iter().cloned());
        ids
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

/// Describes a single socket that should be opened. One Socket unit may contain multiple of these
#[derive(Clone, Debug)]
pub struct SingleSocketConfig {
    pub kind: SocketKind,
    pub specialized: SpecializedSocketConfig,
}

/// All settings from the Exec section of a unit
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
/// The immutable config of a service unit
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
    pub sockets: Vec<UnitId>,
}

/// The immutable config of a socket unit
pub struct SocketConfig {
    pub sockets: Vec<SingleSocketConfig>,
    pub filedesc_name: String,
    pub services: Vec<UnitId>,

    pub exec_config: ExecConfig,
}
