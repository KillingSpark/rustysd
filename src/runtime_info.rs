//! The RuntimeInfo encapsulates all information rustysd needs to do its job. The units, the pid and filedescriptors and the rustysd config.
//! In the lifetime of ruytsd there will only ever be one RuntimeInfo which is passed wrapped inside the ArcMutRuntimeInfo.
//!
//! The idea here is to make as much as possible concurrently readable while still being able to get exclusive access to e.g. remove units.
//! Note that units themselves contain RWLocks so they can be worked on concurrently as long as no write() lock is placed on the RuntimeInfo.

use crate::fd_store::FDStore;
use crate::platform::EventFd;
use crate::units::*;

use nix::unistd::Pid;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

pub type UnitTable = HashMap<UnitId, Unit>;
pub type MutFDStore = RwLock<FDStore>;

/// This will be passed through to all the different threads as a central state struct
pub struct RuntimeInfo {
    pub unit_table: UnitTable,
    pub pid_table: Mutex<PidTable>,
    pub fd_store: MutFDStore,
    pub config: crate::config::Config,
    pub stdout_eventfd: EventFd,
    pub stderr_eventfd: EventFd,
    pub notification_eventfd: EventFd,
    pub socket_activation_eventfd: EventFd,
}

impl RuntimeInfo {
    pub fn notify_eventfds(&self) {
        crate::platform::notify_event_fd(self.stdout_eventfd);
        crate::platform::notify_event_fd(self.stderr_eventfd);
        crate::platform::notify_event_fd(self.notification_eventfd);
        crate::platform::notify_event_fd(self.socket_activation_eventfd);
    }
}

pub type ArcMutRuntimeInfo = Arc<RwLock<RuntimeInfo>>;

/// The PidTable holds info about all launched processes
pub type PidTable = HashMap<Pid, PidEntry>;

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
/// A process can be launched for these reasons. How an exit is handled depends
/// on this reason (e.g. oneshot services are supposed to exit. Normal services should not exit.)
pub enum PidEntry {
    Service(UnitId, ServiceType),
    ServiceExited(crate::signal_handler::ChildTermination),
    Helper(UnitId, String),
    HelperExited(crate::signal_handler::ChildTermination),
}
