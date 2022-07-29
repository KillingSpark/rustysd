//! All the different parts of service starting/killing.
//! 1. Forking
//! 2. processgroupid setting
//! 3. duping of filedescriptors
//! 4. signaling processgroup on kill
mod fork_child;
pub mod fork_os_specific;
mod fork_parent;
mod kill_os_specific;
mod prepare_service;
mod service_exit_handler;
mod services;
mod start_service;
pub use service_exit_handler::*;
pub use services::*;
