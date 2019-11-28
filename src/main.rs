mod services;
mod sockets;
mod unit_parser;
mod units;
use units::*;
mod control;
mod notification_handler;
mod unix_listener_select;
mod start_service;
mod signal_handler;

extern crate signal_hook;

#[macro_use]
extern crate log;
extern crate crossbeam;
extern crate fern;
extern crate lumberjack_rs;
extern crate serde_json;
extern crate threadpool;

use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

fn setup_logging() -> Result<(), String>{
    let lmbrjck_conf = lumberjack_rs::Conf {
        max_age: None,
        max_files: Some(10),
        max_size: 10 * 1024 * 1024,
        log_dir: "./logs".into(),
        name_template: "rustysdlog.log".to_owned(),
    };

    let rotating = std::sync::Mutex::new(lumberjack_rs::new(lmbrjck_conf).unwrap());

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Trace)
        .chain(std::io::stdout())
        .chain(fern::Output::call(move |record| {
            let msg = format!("{}\n", record.args());
            let rotating = rotating.lock();
            let mut rotating = rotating.unwrap();
            let result = rotating.write_all(msg.as_str().as_bytes());
            //TODO do something with the result
            let _ = result;
        }))
        .apply().map_err(|e| format!("Error while stting up logger: {}", e))
}



fn main() {
    setup_logging().unwrap();

    let mut base_id = 0;
    let mut service_table = HashMap::new();
    unit_parser::parse_all_services(
        &mut service_table,
        &PathBuf::from("./test_units"),
        &mut base_id,
    ).unwrap();

    let mut socket_unit_table = HashMap::new();
    unit_parser::parse_all_sockets(
        &mut socket_unit_table,
        &PathBuf::from("./test_units"),
        &mut base_id,
    ).unwrap();

    units::fill_dependencies(&mut service_table);
    for srvc in service_table.values_mut() {
        srvc.dedup_dependencies();
    }

    let service_table = apply_sockets_to_services(service_table, &socket_unit_table).unwrap();

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

    signal_handler::handle_signals(service_table.clone(), socket_table.clone(), pid_table.clone());
}

fn apply_sockets_to_services(
    mut service_table: HashMap<InternalId, Unit>,
    socket_table: &HashMap<InternalId, Unit>,
) -> Result<HashMap<InternalId, Unit>, String> {
    for sock_unit in socket_table.values() {
        let mut counter = 0;
        
        if let UnitSpecialized::Socket(sock) = &sock_unit.specialized {
            trace!("Searching services for socket: {}", sock_unit.conf.name());
            for srvc_unit in service_table.values_mut() {
                let srvc = &mut srvc_unit.specialized;
                if let UnitSpecialized::Service(srvc) = srvc {

                    // add sockets for services with the exact same name
                    if (srvc_unit.conf.name() == sock_unit.conf.name())
                        && !srvc.socket_names.contains(&sock_unit.conf.name())
                    {
                        trace!(
                            "add socket: {} to service: {}",
                            sock_unit.conf.name(),
                            srvc_unit.conf.name()
                        );

                        srvc.socket_names.push(sock.name.clone());
                        counter+=1;
                    }
                    
                    // add sockets to services that specify that the socket belongs to them
                    if let Some(srvc_conf) = &srvc.service_config {
                        if srvc_conf.sockets.contains(&sock_unit.conf.name()) {
                            trace!(
                                "add socket: {} to service: {}",
                                sock_unit.conf.name(),
                                srvc_unit.conf.name()
                            );
                            srvc.socket_names.push(sock.name.clone());
                            counter+=1;
                        }
                    }
                }
            }
            
            // add socket to the specified services
            for srvc_name in &sock.services {
                for srvc_unit in service_table.values_mut() {
                    let srvc = &mut srvc_unit.specialized;
                    if let UnitSpecialized::Service(srvc) = srvc {
                        if (*srvc_name == srvc_unit.conf.name())
                        && !srvc.socket_names.contains(&sock_unit.conf.name())
                        {
                            trace!(
                                "add socket: {} to service: {}",
                                sock_unit.conf.name(),
                                srvc_unit.conf.name()
                            );
                            
                            srvc.socket_names.push(sock.name.clone());
                            counter+=1;
                        }
                    }
                }
            }
        }
        if counter > 1 {
            return Err(format!("Added socket: {} to too many services (should be at most one): {}", sock_unit.conf.name(), counter));
        }
        if counter == 0 {
            warn!("Added socket: {} to no service", sock_unit.conf.name());
        }
    }

    Ok(service_table)
}

