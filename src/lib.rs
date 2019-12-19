//! From the Readme:
//!
//! Rustysd is a service manager that tries to replicate systemd behaviour for a subset of the configuration possibilities.
//! It focuses on the core functionality of a service manager.
//!
//! For now this project is just out of interest how far I could come with this
//! and what would be needed to get a somewhat working system. It is very much a proof of concept / work in progress. For the love of god do not use this
//! in anything that is important.
//! It does look somewhat promising, the core parts are "working" (not thoroughly tested) but there is a lot of cleanup to be done. There is a whole lot of unwrap() calling
//! where error handling should be done properly. It would be a bit unhelpful if your service-manager starts panicing.
//!
//! What is explicitly in scope of this project
//! 1. Startup sorted by dependencies (parallel if possible for unrelated services)
//! 1. Socket activation of services
//! 1. Kill services that have dependencies on failed services
//!
//! What is explicitly out of scope (for now, this project is still very young):
//! 1. Timers
//! 1. Mounts
//! 1. Device
//! 1. Path activation
//! 1. Scopes
//! 1. Slices (this might be added as it is fairly important if you are not running inside of a container)
pub mod config;
pub mod control;
pub mod dbus_wait;
pub mod logging;
pub mod notification_handler;
pub mod services;
pub mod signal_handler;
pub mod sockets;
pub mod units;
pub mod wait_for_socket_activation;

#[macro_use]
extern crate log;
extern crate fern;
extern crate lumberjack_rs;
extern crate serde_json;
extern crate threadpool;
extern crate toml;

#[cfg(target_os = "linux")]
pub fn become_subreaper(set: bool) {
    unsafe {
        // Set subreaper to collect all zombies left behind by the services
        let res = if set {
            libc::prctl(libc::PR_SET_CHILD_SUBREAPER, 1)
        } else {
            libc::prctl(libc::PR_SET_CHILD_SUBREAPER, 0)
        };
        if res < 0 {
            error!("Couldnt set subreaper for rustysd");
            return;
        }
    }
}
#[cfg(any(
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly"
))]
extern "C" {fn procctl(option: c_int, ...) -> c_int;}

#[cfg(any(
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly"
))]
pub fn become_subreaper(set: bool) {
    unsafe {
        // Set subreaper to collect all zombies left behind by the services
        // TODO make pull request to libc to include this
        let PROC_REAP_ACQUIRE = 2;
        let PROC_REAP_RELEASE = 3;
        let res = if set {
            procctl(PROC_REAP_ACQUIRE)
        } else {
            procctl(PROC_REAP_RELEASE)
        };
        if res < 0 {
            error!("Couldnt set subreaper for rustysd");
            return;
        }
    }
}
