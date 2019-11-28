mod control;
mod logging;
mod notification_handler;
mod services;
mod signal_handler;
mod sockets;
mod start_service;
mod unit_parser;
mod units;
mod unix_listener_select;

extern crate signal_hook;

#[macro_use]
extern crate log;
extern crate crossbeam;
extern crate fern;
extern crate lumberjack_rs;
extern crate serde_json;
extern crate threadpool;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

fn main() {
    logging::setup_logging().unwrap();

    let mut base_id = 0;
    let mut service_table = HashMap::new();
    unit_parser::parse_all_services(
        &mut service_table,
        &PathBuf::from("./test_units"),
        &mut base_id,
    )
    .unwrap();

    let mut socket_unit_table = HashMap::new();
    unit_parser::parse_all_sockets(
        &mut socket_unit_table,
        &PathBuf::from("./test_units"),
        &mut base_id,
    )
    .unwrap();

    units::fill_dependencies(&mut service_table);
    for srvc in service_table.values_mut() {
        srvc.dedup_dependencies();
    }

    let service_table =
        sockets::apply_sockets_to_services(service_table, &socket_unit_table).unwrap();

    sockets::open_all_sockets(&mut socket_unit_table).unwrap();

    let pid_table = HashMap::new();
    let (service_table, pid_table) =
        services::run_services(service_table, pid_table, socket_unit_table.clone());

    let service_table = Arc::new(Mutex::new(service_table));
    let pid_table = Arc::new(Mutex::new(pid_table));
    let socket_table = Arc::new(Mutex::new(socket_unit_table));

    notification_handler::handle_notifications(
        socket_table.clone(),
        service_table.clone(),
        pid_table.clone(),
    );

    control::accept_control_connections(service_table.clone(), socket_table.clone());

    signal_handler::handle_signals(
        service_table.clone(),
        socket_table.clone(),
        pid_table.clone(),
    );
}
