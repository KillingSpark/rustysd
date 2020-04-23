//! FDStore is a sort of key-value store that holds open file descriptors.
//! These can come from two sources:
//! 1. Socket units. These are found with the name of their unit (eg "myservice.socket")
//! 1. The sd_notifiy API which can ask rustysd to store some file descriptors so they stay open over restarts
use std::{
    collections::HashMap,
    os::unix::io::{AsRawFd, RawFd},
};

use crate::units::UnitId;

type GlobalEntry = Vec<(UnitId, String, Box<dyn AsRawFd + Send + Sync>)>;

#[derive(Default)]
pub struct FDStore {
    // Indexed by unit name
    global_sockets: HashMap<String, GlobalEntry>,
    service_stored_sockets: HashMap<String, HashMap<String, Vec<Box<RawFd>>>>,
}

impl FDStore {
    pub fn global_fds_to_ids(&self) -> Vec<(RawFd, UnitId)> {
        self.global_sockets
            .values()
            .fold(Vec::new(), |mut acc, fds| {
                for (id, _, fd) in fds {
                    acc.push((fd.as_raw_fd(), id.clone()));
                }
                acc
            })
    }

    /// Insert new fds when opening socket unit. Returns new_fds again if there is already a vec of fds stored.
    /// In this case first get them and close them, then reinsert new_fds again.
    pub fn insert_global(&mut self, name: String, new_fds: GlobalEntry) -> Option<GlobalEntry> {
        if !self.global_sockets.contains_key(&name) {
            self.global_sockets.insert(name, new_fds);
            None
        } else {
            Some(new_fds)
        }
    }

    /// normal remove semantics on a hashmap
    pub fn remove_global(&mut self, name: &String) -> Option<GlobalEntry> {
        self.global_sockets.remove(name)
    }

    /// normal get semantics on a hashmap
    pub fn get_global(&self, name: &str) -> Option<&GlobalEntry> {
        self.global_sockets.get(name)
    }

    /// Insert FDs from the sd_notify API. Indexed by service unit name and the given name for the fds
    pub fn insert_service_stored(
        &mut self,
        srvc_name: String,
        fd_name: String,
        new_fds: Vec<Box<RawFd>>,
    ) {
        self.service_stored_sockets
            .entry(srvc_name)
            .or_insert(HashMap::new())
            .entry(fd_name)
            .or_insert(Vec::new())
            .extend(new_fds);
    }

    /// normal remove semantics on a hashmap
    pub fn remove_service_stored(
        &mut self,
        srvc_name: &String,
        fd_name: &String,
    ) -> Option<Vec<Box<RawFd>>> {
        if let Some(fds) = self.service_stored_sockets.get_mut(srvc_name) {
            fds.remove(fd_name)
        } else {
            None
        }
    }
    /// normal get semantics on a hashmap
    pub fn get_service_stored(
        &self,
        srvc_name: &String,
        fd_name: &String,
    ) -> Option<&Vec<Box<RawFd>>> {
        if let Some(fds) = self.service_stored_sockets.get(srvc_name) {
            fds.get(fd_name)
        } else {
            None
        }
    }
}
