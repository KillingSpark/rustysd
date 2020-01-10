//! This module provides the control access similar to systemctl from systemd. It uses the jsonrpc 2.0 spec and has the interface defined below.
//!
//! ### list-units Option<kind>
//! Kind either "target", "socket", "service"
//! Give no kind to list all units of all types
//! Lists all units. In the future there should be a filtering mechanism for type / name-matching / etc...
//!
//! ### status Option<name>
//! * If the param is a string show status of the unit with that name (might get the same filtering as list-units in the future).
//! * If no param is given, show status of all units
//! 
//! ### restart name
//! Restart unit with that name. If it was running first kill it. If it is already stopped start it.

//! ### stop name
//! Stop unit with that name. Will recursivly stop all units that require that unit

//! ### enable name
//! Load new file with that name. Useful if you moved/copied a file in the unit-dirs and want to start it without restarting rustysd as a whole
//! 
//! ### shutdown
//! Shutdown rustysd by killing all services, closing all sockets and exiting
//!
//! ## Send commands
//! Currently there is no utility to send commands to this service. There will be in the future. Until then this can be used to send
//! calls to the control socket
//! echo '{"method": "restart", "params": "test.service"}' | socat - TCP-CONNECT:0.0.0.0:8080

mod control;
pub mod jsonrpc2;

pub use control::*;
