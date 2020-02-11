//! This module provides the control access similar to systemctl from systemd. It uses the jsonrpc 2.0 spec and has the interface defined in doc/ControlInterface.md

mod control;
pub mod jsonrpc2;

pub use control::*;
