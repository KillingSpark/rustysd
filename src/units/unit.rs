use log::trace;

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

impl SocketState {
    fn activate(
        &mut self,
        id: &UnitId,
        conf: &SocketConfig,
        status: &RwLock<UnitStatus>,
        run_info: &RuntimeInfo,
    ) -> Result<UnitStatus, UnitOperationError> {
        let open_res = self
            .sock
            .open_all(
                conf,
                id.name.clone(),
                id.clone(),
                &mut *run_info.fd_store.write().unwrap(),
            )
            .map_err(|e| UnitOperationError {
                unit_name: id.name.clone(),
                unit_id: id.clone(),
                reason: UnitOperationErrorReason::SocketOpenError(format!("{}", e)),
            });
        match open_res {
            Ok(_) => {
                let mut status = status.write().unwrap();
                *status = UnitStatus::Started(StatusStarted::Running);
                run_info.notify_eventfds();
                Ok(UnitStatus::Started(StatusStarted::Running))
            }
            Err(e) => {
                let mut status = status.write().unwrap();
                *status =
                    UnitStatus::Stopped(StatusStopped::StoppedUnexpected, vec![e.reason.clone()]);
                Err(e)
            }
        }
    }

    fn deactivate(
        &mut self,
        id: &UnitId,
        conf: &SocketConfig,
        status: &RwLock<UnitStatus>,
        run_info: &RuntimeInfo,
    ) -> Result<(), UnitOperationError> {
        let close_result = self
            .sock
            .close_all(
                &conf,
                id.name.clone(),
                &mut *run_info.fd_store.write().unwrap(),
            )
            .map_err(|e| UnitOperationError {
                unit_name: id.name.clone(),
                unit_id: id.clone(),
                reason: UnitOperationErrorReason::SocketCloseError(e),
            });
        match &close_result {
            Ok(_) => {
                let mut status = status.write().unwrap();
                *status = UnitStatus::Stopped(StatusStopped::StoppedFinal, vec![]);
            }
            Err(e) => {
                let mut status = status.write().unwrap();
                *status = UnitStatus::Stopped(StatusStopped::StoppedFinal, vec![e.reason.clone()]);
            }
        };
        close_result
    }

    fn reactivate(
        &mut self,
        id: &UnitId,
        conf: &SocketConfig,
        status: &RwLock<UnitStatus>,
        run_info: &RuntimeInfo,
    ) -> Result<(), UnitOperationError> {
        let close_result = self
            .sock
            .close_all(
                &conf,
                id.name.clone(),
                &mut *run_info.fd_store.write().unwrap(),
            )
            .map_err(|e| UnitOperationError {
                unit_name: id.name.clone(),
                unit_id: id.clone(),
                reason: UnitOperationErrorReason::SocketCloseError(e),
            });

        // If closing failed, dont try to restart but fail early
        if let Err(error) = close_result {
            let mut status = status.write().unwrap();
            *status = UnitStatus::Stopped(StatusStopped::StoppedFinal, vec![error.reason.clone()]);
            return Err(error);
        }

        // Reopen and set the status according to the result
        let open_res = self
            .sock
            .open_all(
                conf,
                id.name.clone(),
                id.clone(),
                &mut *run_info.fd_store.write().unwrap(),
            )
            .map_err(|e| UnitOperationError {
                unit_name: id.name.clone(),
                unit_id: id.clone(),
                reason: UnitOperationErrorReason::SocketOpenError(format!("{}", e)),
            });
        match open_res {
            Ok(_) => {
                let mut status = status.write().unwrap();
                *status = UnitStatus::Started(StatusStarted::Running);
                run_info.notify_eventfds();
                Ok(())
            }
            Err(e) => {
                let mut status = status.write().unwrap();
                *status =
                    UnitStatus::Stopped(StatusStopped::StoppedUnexpected, vec![e.reason.clone()]);
                Err(e)
            }
        }
    }
}

impl ServiceState {
    fn activate(
        &mut self,
        id: &UnitId,
        conf: &ServiceConfig,
        status: &RwLock<UnitStatus>,
        run_info: &RuntimeInfo,
        source: ActivationSource,
    ) -> Result<UnitStatus, UnitOperationError> {
        let start_res = self
            .srvc
            .start(conf, id.clone(), &id.name, run_info, source)
            .map_err(|e| UnitOperationError {
                unit_name: id.name.clone(),
                unit_id: id.clone(),
                reason: UnitOperationErrorReason::ServiceStartError(e),
            });
        match start_res {
            Ok(crate::services::StartResult::Started) => {
                {
                    let mut status = status.write().unwrap();
                    *status = UnitStatus::Started(StatusStarted::Running);
                }
                Ok(UnitStatus::Started(StatusStarted::Running))
            }
            Ok(crate::services::StartResult::WaitingForSocket) => {
                {
                    let mut status = status.write().unwrap();
                    *status = UnitStatus::Started(StatusStarted::WaitingForSocket);
                }
                // tell socket activation to listen to these sockets again
                for socket_id in &conf.sockets {
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
                let mut status = status.write().unwrap();
                *status =
                    UnitStatus::Stopped(StatusStopped::StoppedUnexpected, vec![e.reason.clone()]);
                Err(e)
            }
        }
    }

    fn deactivate(
        &mut self,
        id: &UnitId,
        conf: &ServiceConfig,
        status: &RwLock<UnitStatus>,
        run_info: &RuntimeInfo,
    ) -> Result<(), UnitOperationError> {
        let kill_result = self
            .srvc
            .kill(&conf, id.clone(), &id.name, run_info)
            .map_err(|e| UnitOperationError {
                unit_name: id.name.clone(),
                unit_id: id.clone(),
                reason: UnitOperationErrorReason::ServiceStopError(e),
            });
        match &kill_result {
            Ok(_) => {
                let mut status = status.write().unwrap();
                *status = UnitStatus::Stopped(StatusStopped::StoppedFinal, vec![]);
            }
            Err(e) => {
                let mut status = status.write().unwrap();
                *status = UnitStatus::Stopped(StatusStopped::StoppedFinal, vec![e.reason.clone()]);
            }
        }
        kill_result
    }
    fn reactivate(
        &mut self,
        id: &UnitId,
        conf: &ServiceConfig,
        status: &RwLock<UnitStatus>,
        run_info: &RuntimeInfo,
        source: ActivationSource,
    ) -> Result<(), UnitOperationError> {
        let kill_result = self
            .srvc
            .kill(&conf, id.clone(), &id.name, run_info)
            .map_err(|e| UnitOperationError {
                unit_name: id.name.clone(),
                unit_id: id.clone(),
                reason: UnitOperationErrorReason::ServiceStopError(e),
            });

        // If killing failed, dont try to restart but fail early
        if let Err(error) = kill_result {
            let mut status = status.write().unwrap();
            *status = UnitStatus::Stopped(StatusStopped::StoppedFinal, vec![error.reason.clone()]);
            return Err(error);
        }

        // Restart and set the status according to the result
        let start_res = self
            .srvc
            .start(conf, id.clone(), &id.name, run_info, source)
            .map_err(|e| UnitOperationError {
                unit_name: id.name.clone(),
                unit_id: id.clone(),
                reason: UnitOperationErrorReason::ServiceStartError(e),
            });
        match start_res {
            Ok(crate::services::StartResult::Started) => {
                {
                    let mut status = status.write().unwrap();
                    *status = UnitStatus::Started(StatusStarted::Running);
                }
                Ok(())
            }
            Ok(crate::services::StartResult::WaitingForSocket) => {
                {
                    let mut status = status.write().unwrap();
                    *status = UnitStatus::Started(StatusStarted::WaitingForSocket);
                }
                // tell socket activation to listen to these sockets again
                for socket_id in &conf.sockets {
                    if let Some(unit) = run_info.unit_table.get(socket_id) {
                        if let Specific::Socket(sock) = &unit.specific {
                            let mut_state = &mut *sock.state.write().unwrap();
                            mut_state.sock.activated = false;
                        }
                    }
                }
                run_info.notify_eventfds();
                Ok(())
            }
            Err(e) => {
                let mut status = status.write().unwrap();
                *status =
                    UnitStatus::Stopped(StatusStopped::StoppedUnexpected, vec![e.reason.clone()]);
                Err(e)
            }
        }
    }
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

pub struct TargetSpecific {
    pub state: RwLock<TargetState>,
}

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

enum LockedState<'a> {
    Service(
        std::sync::RwLockWriteGuard<'a, ServiceState>,
        &'a ServiceConfig,
    ),
    Socket(
        std::sync::RwLockWriteGuard<'a, SocketState>,
        &'a SocketConfig,
    ),
    Target(std::sync::RwLockWriteGuard<'a, TargetState>),
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

    /// Check if the transition to state 'Starting' can be done
    ///
    /// This is the case if:
    /// 1. All units that have a before relation to this unit have been run at least once
    /// 1. All of the above that are required by this unit are in the state 'Started'
    fn state_transition_starting(&self, run_info: &RuntimeInfo) -> Result<(), Vec<UnitId>> {
        let (mut self_lock, others) = aquire_locks(
            vec![self.id.clone()],
            self.common.dependencies.after.clone(),
            &run_info.unit_table,
        );

        let unstarted_deps = others
            .iter()
            .fold(Vec::new(), |mut acc, (id, status_locked)| {
                let required = self.common.dependencies.requires.contains(id);
                let ready = if required {
                    status_locked.is_started()
                } else {
                    **status_locked != UnitStatus::NeverStarted
                };

                if !ready {
                    acc.push(id.clone());
                }
                acc
            });

        if unstarted_deps.is_empty() {
            **self_lock.get_mut(&self.id).unwrap() = UnitStatus::Starting;
            Ok(())
        } else {
            Err(unstarted_deps)
        }
        // All locks are released again here
    }

    /// Check if the transition to state 'Restarting' can be done. Returns whether the status before was
    /// Started, which requires a full restart.
    ///
    /// This is the case if:
    /// 1. All units that have a before relation to this unit have been run at least once
    /// 1. All of the above that are required by this unit are in the state 'Started'
    fn state_transition_restarting(&self, run_info: &RuntimeInfo) -> Result<bool, Vec<UnitId>> {
        let (mut self_lock, others) = aquire_locks(
            vec![self.id.clone()],
            self.common.dependencies.after.clone(),
            &run_info.unit_table,
        );

        let unstarted_deps = others
            .iter()
            .fold(Vec::new(), |mut acc, (id, status_locked)| {
                let required = self.common.dependencies.requires.contains(id);
                let ready = if required {
                    status_locked.is_started()
                } else {
                    **status_locked != UnitStatus::NeverStarted
                };

                if !ready {
                    acc.push(id.clone());
                }
                acc
            });

        if unstarted_deps.is_empty() {
            let need_full_restart = self_lock.get_mut(&self.id).unwrap().is_started();
            **self_lock.get_mut(&self.id).unwrap() = UnitStatus::Restarting;
            Ok(need_full_restart)
        } else {
            Err(unstarted_deps)
        }
        // All locks are released again here
    }

    /// Check if the transition to state 'Stopping' can be done
    ///
    /// This is the case if:
    /// 1. All units that have a requires relation to this unit have been stopped
    fn state_transition_stopping(&self, run_info: &RuntimeInfo) -> Result<(), Vec<UnitId>> {
        let (mut self_lock, others) = aquire_locks(
            vec![self.id.clone()],
            self.common.dependencies.kill_before_this(),
            &run_info.unit_table,
        );

        let unkilled_depending = others
            .iter()
            .fold(Vec::new(), |mut acc, (id, status_locked)| {
                if status_locked.is_started() {
                    acc.push(id.clone());
                }
                acc
            });

        if unkilled_depending.is_empty() {
            **self_lock.get_mut(&self.id).unwrap() = UnitStatus::Stopping;
            Ok(())
        } else {
            Err(unkilled_depending)
        }
        // All locks are released again here
    }

    /// This activates the unit and manages the state transitions. It reports back the new unit status or any
    /// errors encountered while starting the unit. Note that these errors are also recorded in the units status.
    pub fn activate(
        &self,
        run_info: &RuntimeInfo,
        source: ActivationSource,
    ) -> Result<UnitStatus, UnitOperationError> {
        let state = match &self.specific {
            Specific::Service(specific) => {
                LockedState::Service(specific.state.write().unwrap(), &specific.conf)
            }
            Specific::Socket(specific) => {
                LockedState::Socket(specific.state.write().unwrap(), &specific.conf)
            }
            Specific::Target(specific) => LockedState::Target(specific.state.write().unwrap()),
        };

        {
            let self_status = &*self.common.status.read().unwrap();
            match self_status {
                UnitStatus::Started(StatusStarted::WaitingForSocket) => {
                    if source == ActivationSource::SocketActivation {
                        // Need activation
                    } else {
                        // Dont need activation
                        return Ok(self_status.clone());
                    }
                }
                UnitStatus::Started(_) => {
                    // Dont need activation
                    return Ok(self_status.clone());
                }
                UnitStatus::Stopped(_, _) => {
                    if source == ActivationSource::SocketActivation {
                        // Dont need activation
                        return Ok(self_status.clone());
                    } else {
                        // Need activation
                    }
                }
                _ => {
                    // Need activation
                }
            }
        }

        self.state_transition_starting(run_info).map_err(|bad_ids| {
            trace!(
                "Unit: {} ignores activation. Not all dependencies have been started (still waiting for: {:?})",
                self.id.name,
                bad_ids,
            );
            UnitOperationError {
                reason: UnitOperationErrorReason::DependencyError(bad_ids),
                unit_name: self.id.name.clone(),
                unit_id: self.id.clone(),
            }
        })?;

        match state {
            LockedState::Target(_state) => {
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
            LockedState::Socket(mut state, conf) => {
                let state = &mut *state;
                state.activate(&self.id, conf, &self.common.status, run_info)
            }
            LockedState::Service(mut state, conf) => {
                let state = &mut *state;
                state.activate(&self.id, conf, &self.common.status, run_info, source)
            }
        }
    }

    /// This dectivates the unit and manages the state transitions. It reports back any
    /// errors encountered while stopping the unit
    pub fn deactivate(&self, run_info: &RuntimeInfo) -> Result<(), UnitOperationError> {
        let state = match &self.specific {
            Specific::Service(specific) => {
                LockedState::Service(specific.state.write().unwrap(), &specific.conf)
            }
            Specific::Socket(specific) => {
                LockedState::Socket(specific.state.write().unwrap(), &specific.conf)
            }
            Specific::Target(specific) => LockedState::Target(specific.state.write().unwrap()),
        };

        {
            let self_status = &*self.common.status.read().unwrap();
            match self_status {
                UnitStatus::Stopped(_, _) => {
                    return Ok(());
                }
                _ => {
                    // Need deactivation
                }
            }
        }

        self.state_transition_stopping(run_info).map_err(|bad_ids| {
            trace!(
                "Unit: {} ignores deactivation. Not all units depending on this unit have been started (still waiting for: {:?})",
                self.id.name,
                bad_ids,
            );
            UnitOperationError {
                reason: UnitOperationErrorReason::DependencyError(bad_ids),
                unit_name: self.id.name.clone(),
                unit_id: self.id.clone(),
            }
        })?;

        trace!("Deactivate unit: {}", self.id.name);
        match state {
            LockedState::Target(_) => {
                let mut status = self.common.status.write().unwrap();
                *status = UnitStatus::Stopped(StatusStopped::StoppedFinal, vec![]);
                Ok(())
            }
            LockedState::Socket(mut state, conf) => {
                let state = &mut *state;
                state.deactivate(&self.id, conf, &self.common.status, run_info)
            }
            LockedState::Service(mut state, conf) => {
                let state = &mut *state;
                state.deactivate(&self.id, conf, &self.common.status, run_info)
            }
        }
    }

    /// This rectivates the unit and manages the state transitions. It reports back any
    /// errors encountered while stopping the unit.
    ///
    /// If the unit was stopped this just calls activate.
    pub fn reactivate(
        &self,
        run_info: &RuntimeInfo,
        source: ActivationSource,
    ) -> Result<(), UnitOperationError> {
        trace!("Reactivate unit: {}", self.id.name);

        let state = match &self.specific {
            Specific::Service(specific) => {
                LockedState::Service(specific.state.write().unwrap(), &specific.conf)
            }
            Specific::Socket(specific) => {
                LockedState::Socket(specific.state.write().unwrap(), &specific.conf)
            }
            Specific::Target(specific) => LockedState::Target(specific.state.write().unwrap()),
        };

        let need_full_restart = self.state_transition_restarting(run_info).map_err(|bad_ids| {
            trace!(
                "Unit: {} ignores deactivation. Not all units depending on this unit have been started (still waiting for: {:?})",
                self.id.name,
                bad_ids,
            );
            UnitOperationError {
                reason: UnitOperationErrorReason::DependencyError(bad_ids),
                unit_name: self.id.name.clone(),
                unit_id: self.id.clone(),
            }
        })?;

        if need_full_restart {
            match state {
                LockedState::Target(_) => {
                    let mut status = self.common.status.write().unwrap();
                    *status = UnitStatus::Started(StatusStarted::Running);
                    Ok(())
                }
                LockedState::Socket(mut state, conf) => {
                    let state = &mut *state;
                    state.reactivate(&self.id, conf, &self.common.status, run_info)
                }
                LockedState::Service(mut state, conf) => {
                    let state = &mut *state;
                    state.reactivate(&self.id, conf, &self.common.status, run_info, source)
                }
            }
        } else {
            match state {
                LockedState::Target(_) => {
                    let mut status = self.common.status.write().unwrap();
                    *status = UnitStatus::Started(StatusStarted::Running);
                    Ok(())
                }
                LockedState::Socket(mut state, conf) => {
                    let state = &mut *state;
                    state
                        .activate(&self.id, conf, &self.common.status, run_info)
                        .map(|_| ())
                }
                LockedState::Service(mut state, conf) => {
                    let state = &mut *state;
                    state
                        .activate(&self.id, conf, &self.common.status, run_info, source)
                        .map(|_| ())
                }
            }
        }
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
    pub fn start_before_this(&self) -> Vec<UnitId> {
        let mut ids = Vec::new();
        ids.extend(self.after.iter().cloned());
        ids
    }
    pub fn start_concurrently_with_this(&self) -> Vec<UnitId> {
        let mut ids = Vec::new();
        ids.extend(self.wants.iter().cloned());
        ids.extend(self.requires.iter().cloned());
        let ids = ids
            .into_iter()
            .filter(|id| !self.after.contains(&id))
            .collect();
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
    pub environment: Option<EnvVars>,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Eq, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct PlatformSpecificServiceFields {
    pub cgroup_path: std::path::PathBuf,
}

#[cfg(not(target_os = "linux"))]
#[derive(Clone, Eq, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
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
