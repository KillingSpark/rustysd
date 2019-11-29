mod control;
mod logging;
mod notification_handler;
mod services;
mod signal_handler;
mod sockets;
mod start_service;
mod unit_parser;
mod units;

extern crate signal_hook;

#[macro_use]
extern crate log;
extern crate crossbeam;
extern crate fern;
extern crate lumberjack_rs;
extern crate serde_json;
extern crate threadpool;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

fn main() {
    logging::setup_logging().unwrap();

    // initial loading of the units and matching of the various before/after settings
    // also opening all fildescriptors in the socket files
    let (service_table, socket_unit_table) =
        unit_parser::load_all_units(&PathBuf::from("./test_units")).unwrap();

    // parallel startup of all services
    let (service_table, pid_table) =
        services::run_services(service_table, socket_unit_table.clone());

    // wrapping in arc<mutex<>> to share between the various threads
    let service_table = Arc::new(Mutex::new(service_table));
    let pid_table = Arc::new(Mutex::new(pid_table));
    let socket_table = Arc::new(Mutex::new(socket_unit_table));

    // listen on user commands like listunits/kill/restart...
    control::accept_control_connections(service_table.clone(), socket_table.clone());

    // listen on signals from the child processes
    signal_handler::handle_signals(
        service_table.clone(),
        socket_table.clone(),
        pid_table.clone(),
    );
}
