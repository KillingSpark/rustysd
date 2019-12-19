//! This module provides the control access similar to systemctl from systemd. It uses the jsonrpc 2.0 spec and has the interface defined below.
//!
//! ### list-units
//! Lists all units. In the future there should be a filtering mechanism for type / name-matching / etc...
//!
//! ### status
//! * If the param is a string show status of the unit with that name (might get the same filtering as list-units in the future).
//! * If no param is given, show status of all units
//!
//! ## Send commands
//! Currently there is no utility to send commands to this service. There will be in the future. Until then this can be used to send
//! calls to the control socket
//! echo '{"method": "restart", "params": "test.service"}' | socat - TCP-CONNECT:0.0.0.0:8080

mod control;
mod jsonrpc2;

pub use control::*;
