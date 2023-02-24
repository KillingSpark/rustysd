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
pub mod entrypoints;
pub mod fd_store;
pub mod logging;
pub mod notification_handler;
pub mod platform;
pub mod runtime_info;
pub mod services;
pub mod shutdown;
pub mod signal_handler;
pub mod socket_activation;
pub mod sockets;
pub mod units;

#[cfg(test)]
mod tests;
