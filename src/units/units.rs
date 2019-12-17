use crate::services::Service;
use crate::sockets::{Socket, SocketKind, SpecializedSocketConfig};
use std::os::unix::io::AsRawFd;
use std::os::unix::io::RawFd;

use nix::unistd::Pid;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::{fmt, path::PathBuf};

pub type InternalId = u64;
pub type SocketTable = HashMap<InternalId, Unit>;
pub type ArcMutSocketTable = Arc<Mutex<SocketTable>>;

pub type ServiceTable = HashMap<InternalId, Unit>;
pub type ArcMutServiceTable = Arc<Mutex<ServiceTable>>;

pub type UnitTable = HashMap<InternalId, Arc<Mutex<Unit>>>;
pub type ArcMutUnitTable = Arc<Mutex<UnitTable>>;

pub type PidTable = HashMap<Pid, PidEntry>;
pub type ArcMutPidTable = Arc<Mutex<PidTable>>;

#[derive(Eq, PartialEq, Hash)]
pub enum PidEntry {
    Service(InternalId),
    Stop(InternalId),
}

// TODO delete this
// keep around while refactoring in case it is needed again
#[allow(dead_code)]
pub fn find_sock_with_name<'b>(name: &str, sockets: &'b SocketTable) -> Option<&'b Socket> {
    let sock: Vec<&'b Socket> = sockets
        .iter()
        .map(|(_id, unit)| {
            if let UnitSpecialized::Socket(sock) = &unit.specialized {
                Some(sock)
            } else {
                None
            }
        })
        .filter(|sock| match sock {
            Some(sock) => sock.name == *name,
            None => false,
        })
        .map(std::option::Option::unwrap)
        .collect();
    if sock.len() == 1 {
        Some(sock[0])
    } else {
        None
    }
}

#[derive(Debug)]
pub enum UnitSpecialized {
    Socket(Socket),
    Service(Service),
}

#[derive(Debug, Default)]
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
    pub fn dedup_dependencies(&mut self) {
        self.install.wants.dedup();
        self.install.requires.dedup();
        self.install.wanted_by.dedup();
        self.install.required_by.dedup();
        self.install.before.dedup();
        self.install.after.dedup();
    }

    pub fn activate(
        &mut self,
        sockets: &HashMap<InternalId, &crate::sockets::Socket>,
        pids: ArcMutPidTable,
        notification_socket_path: std::path::PathBuf,
        eventfds: &[RawFd],
    ) -> Result<(), String> {
        match &mut self.specialized {
            UnitSpecialized::Socket(sock) => {
                sock.open_all().map_err(|e| format!("Error opening socket {}: {}", self.conf.name(), e))?;
            }
            UnitSpecialized::Service(srvc) => {
                srvc.start(
                    self.id,
                    &self.conf.name(),
                    sockets,
                    pids,
                    notification_socket_path,
                    eventfds,
                );
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

pub fn fill_dependencies(units: &mut HashMap<InternalId, Unit>) {
    let mut name_to_id = HashMap::new();

    for (id, unit) in &*units {
        let name = unit.conf.name();
        name_to_id.insert(name, *id);
    }

    let mut required_by = Vec::new();
    let mut wanted_by: Vec<(InternalId, InternalId)> = Vec::new();
    let mut before = Vec::new();
    let mut after = Vec::new();

    for unit in (*units).values_mut() {
        let conf = &unit.conf;
        for name in &conf.wants {
            let id = name_to_id[name.as_str()];
            unit.install.wants.push(id);
            wanted_by.push((id, unit.id));
        }
        for name in &conf.requires {
            let id = name_to_id[name.as_str()];
            unit.install.requires.push(id);
            required_by.push((id, unit.id));
        }
        for name in &conf.before {
            let id = name_to_id[name.as_str()];
            unit.install.before.push(id);
            after.push((unit.id, id))
        }
        for name in &conf.after {
            let id = name_to_id[name.as_str()];
            unit.install.after.push(id);
            before.push((unit.id, id))
        }

        if let Some(conf) = &unit.install.install_config {
            for name in &conf.wanted_by {
                let id = name_to_id[name.as_str()];
                wanted_by.push((unit.id, id));
            }
        }
        if let Some(conf) = &unit.install.install_config {
            for name in &conf.required_by {
                let id = name_to_id[name.as_str()];
                required_by.push((unit.id, id));
            }
        }
    }

    for (wanted, wanting) in wanted_by {
        let unit = units.get_mut(&wanting).unwrap();
        unit.install.wants.push(wanted);
        let unit = units.get_mut(&wanted).unwrap();
        unit.install.wanted_by.push(wanting);
    }

    for (required, requiring) in required_by {
        let unit = units.get_mut(&requiring).unwrap();
        unit.install.requires.push(required);
        let unit = units.get_mut(&required).unwrap();
        unit.install.required_by.push(requiring);
    }

    for (before, after) in before {
        let unit = units.get_mut(&after).unwrap();
        unit.install.before.push(before);
    }
    for (after, before) in after {
        let unit = units.get_mut(&before).unwrap();
        unit.install.after.push(after);
    }

    for srvc in units.values_mut() {
        srvc.dedup_dependencies();
    }
}
