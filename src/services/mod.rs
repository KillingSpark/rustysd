//! All the different parts of service starting/killing.
//! 1. Forking
//! 2. processgroupid setting
//! 3. duping of filedescriptors
//! 4. signaling processgroup on kill
mod fork_child;
mod fork_parent;
mod pre_fork;
mod pre_fork_os_specific;
mod services;
mod start_service;
pub use services::*;
