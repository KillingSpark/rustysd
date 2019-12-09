use std::os::unix::io::AsRawFd;
use crate::services::Service;
use crate::sockets::{Socket, SocketKind, SpecializedSocketConfig};

use nix::unistd::Pid;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub type InternalId = u64;
pub type SocketTable = HashMap<InternalId, Unit>;
pub type ArcMutSocketTable = Arc<Mutex<SocketTable>>;

pub type ServiceTable = HashMap<InternalId, Unit>;
pub type ArcMutServiceTable = Arc<Mutex<ServiceTable>>;

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

pub fn get_sockets_by_name<'b>(socket_units: &'b SocketTable) -> HashMap<String, &'b Socket> {
    let mut sockets = HashMap::new();

    for sock_unit in socket_units.values() {
        if let UnitSpecialized::Socket(sock) = &sock_unit.specialized {
            sockets.insert(sock.name.clone(), sock);
        }
    }

    sockets
}

pub enum UnitSpecialized {
    Socket(Socket),
    Service(Service),
}

#[derive(Default)]
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
}

pub struct UnitConfig {
    pub filepath: PathBuf,

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

pub struct SocketConfig {
    pub kind: SocketKind,
    pub specialized: SpecializedSocketConfig,

    pub fd: Option<Arc<Box<dyn AsRawFd>>>,
}

unsafe impl Send for SocketConfig {}

pub struct InstallConfig {
    pub wanted_by: Vec<String>,
    pub required_by: Vec<String>,
}

pub enum ServiceType {
    Simple,
    Notify,
}

pub enum NotifyKind {
    Main,
    Exec,
    All,
    None,
}

pub struct ServiceConfig {
    pub keep_alive: bool,
    pub notifyaccess: NotifyKind,
    pub exec: String,
    pub stop: String,
    pub srcv_type: ServiceType,

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
