use crate::platform::EventFd;
use crate::services::Service;
use crate::sockets::{Socket, SocketKind, SpecializedSocketConfig};
use std::os::unix::io::AsRawFd;

use nix::unistd::Pid;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::{fmt, path::PathBuf};

pub type InternalId = u64;
pub type SocketTable = HashMap<InternalId, Unit>;
pub type ServiceTable = HashMap<InternalId, Unit>;
pub type TargetTable = HashMap<InternalId, Unit>;

pub type UnitTable = HashMap<InternalId, Arc<Mutex<Unit>>>;
pub type ArcMutUnitTable = Arc<RwLock<UnitTable>>;

pub type PidTable = HashMap<Pid, PidEntry>;
pub type ArcMutPidTable = Arc<Mutex<PidTable>>;

#[derive(Eq, PartialEq, Hash)]
pub enum PidEntry {
    Service(InternalId),
    Stop(InternalId),
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
    pub wants: Vec<InternalId>,
    pub requires: Vec<InternalId>,

    pub wanted_by: Vec<InternalId>,
    pub required_by: Vec<InternalId>,

    pub before: Vec<InternalId>,
    pub after: Vec<InternalId>,

    pub install_config: Option<InstallConfig>,
}

pub struct Unit {
    pub id: InternalId,
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

    fn ids_needed_for_activation(&self) -> Vec<InternalId> {
        match &self.specialized {
            UnitSpecialized::Target => Vec::new(),
            UnitSpecialized::Socket(_) => Vec::new(),
            UnitSpecialized::Service(srvc) => srvc.socket_ids.clone(),
        }
    }

    pub fn filter_units_needed_for_activation(&self, unit_table: &UnitTable) -> UnitTable {
        let ids_needed = self.ids_needed_for_activation();
        let units_needed = unit_table
            .iter()
            .fold(HashMap::new(), |mut acc, (id, unit)| {
                if ids_needed.contains(id) {
                    acc.insert(*id, Arc::clone(unit));
                }
                acc
            });

        units_needed
    }

    pub fn activate(
        &mut self,
        required_units: &mut HashMap<InternalId, &mut Unit>,
        pids: ArcMutPidTable,
        notification_socket_path: std::path::PathBuf,
        eventfds: &[EventFd],
        allow_ignore: bool,
    ) -> Result<(), String> {
        match &mut self.specialized {
            UnitSpecialized::Target => trace!("Reached target {}", self.conf.name()),
            UnitSpecialized::Socket(sock) => {
                sock.open_all()
                    .map_err(|e| format!("Error opening socket {}: {}", self.conf.name(), e))?;
            }
            UnitSpecialized::Service(srvc) => {
                let mut sockets: HashMap<InternalId, &mut Socket> = HashMap::new();
                for (id, unit_locked) in required_units {
                    if let UnitSpecialized::Socket(sock) = &mut unit_locked.specialized {
                        sockets.insert(*id, sock);
                    }
                }
                srvc.start(
                    self.id,
                    &self.conf.name(),
                    &mut sockets,
                    pids,
                    notification_socket_path,
                    eventfds,
                    allow_ignore,
                )?;
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

    pub fd: Option<Arc<Box<dyn AsRawFd>>>,
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

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ServiceType {
    Simple,
    Notify,
    Dbus,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum NotifyKind {
    Main,
    Exec,
    All,
    None,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ServiceConfig {
    pub keep_alive: bool,
    pub accept: bool,
    pub notifyaccess: NotifyKind,
    pub exec: String,
    pub stop: String,
    pub srcv_type: ServiceType,

    pub dbus_name: Option<String>,

    pub sockets: Vec<String>,
}
