use crate::sockets::{Socket, SocketKind, SpecializedSocketConfig};
use crate::services::Service;

use std::path::PathBuf;
use std::collections::HashMap;

pub type InternalId = u64;

#[derive(Clone)]
pub enum UnitSpecialized {
    Socket(Socket),
    Service(Service),
}

#[derive(Default, Clone)]
pub struct Install {
    pub wants: Vec<InternalId>,
    pub requires: Vec<InternalId>,

    pub wanted_by: Vec<InternalId>,
    pub required_by: Vec<InternalId>,

    pub before: Vec<InternalId>,
    pub after: Vec<InternalId>,

    pub install_config: Option<InstallConfig>,
}

#[derive(Clone)]
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

#[derive(Clone)]
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

        let split: Vec<_> = name.split(".").collect();
        let name = split[0..split.len()-1].join(".");

        name
    }
}


#[derive(Clone)]
pub struct SocketConfig {
    pub name: String,
    pub kind: SocketKind,
    pub specialized: SpecializedSocketConfig,
}

#[derive(Clone)]
pub struct InstallConfig {
    pub wanted_by: Vec<String>,
    pub required_by: Vec<String>,
}

#[derive(Clone)]
pub struct ServiceConfig {
    pub keep_alive: bool,
    pub exec: String,
    pub stop: String,
}

pub fn fill_dependencies(units: &mut HashMap<InternalId, Unit>) -> HashMap<String, u64> {
    let mut name_to_id = HashMap::new();

    for (id, unit) in &*units {
        let name = unit.conf.name();
        trace!("Added id for name: {}", name);
        name_to_id.insert(name, *id);
    }

    let mut required_by = Vec::new();
    let mut wanted_by: Vec<(InternalId, InternalId)> = Vec::new();
    let mut before = Vec::new();
    let mut after = Vec::new();

    for (_, unit) in &mut *units {
        let conf = &unit.conf;
        for name in &conf.wants {
            let id = name_to_id.get(name.as_str()).unwrap();
            unit.install.wants.push(*id);
            wanted_by.push((*id, unit.id));
        }
        for name in &conf.requires {
            let id = name_to_id.get(name.as_str()).expect(&format!("Name {} had no matching id", name));
            unit.install.requires.push(*id);
            required_by.push((*id, unit.id));
        }
        for name in &conf.before {
            let id = name_to_id.get(name.as_str()).unwrap();
            unit.install.before.push(*id);
            after.push((unit.id, *id))
        }
        for name in &conf.after {
            let id = name_to_id.get(name.as_str()).unwrap();
            unit.install.after.push(*id);
            before.push((unit.id, *id))
        }

        if let Some(conf) = &unit.install.install_config {
            for name in &conf.wanted_by {
                let id = name_to_id.get(name.as_str()).unwrap();
                wanted_by.push((unit.id, *id));
            }
        }
        if let Some(conf) = &unit.install.install_config {
            for name in &conf.required_by {
                let id = name_to_id.get(name.as_str()).unwrap();
                required_by.push((unit.id, *id));
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

    name_to_id
}