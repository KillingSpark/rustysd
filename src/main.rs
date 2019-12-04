mod config;
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

fn main() {
    let (log_conf, conf) = config::load_config(None);
    logging::setup_logging(&log_conf.log_dir).unwrap();
    let conf = match conf {
        Ok(conf) => conf,
        Err(e) => {
            error!("Error while loading the conf: {}", e);
            panic!(
                "Reading conf did not work. See stdout or log at: {:?}",
                log_conf.log_dir
            );
        }
    };

    // initial loading of the units and matching of the various before/after settings
    // also opening all fildescriptors in the socket files
    let (service_table, socket_unit_table) = unit_parser::load_all_units(&conf.unit_dirs).unwrap();

    // parallel startup of all services
    let (service_table, socket_table, pid_table) = services::run_services(
        service_table,
        socket_unit_table,
        conf.notification_sockets_dir.clone(),
    );

    // listen on user commands like listunits/kill/restart...
    control::accept_control_connections(service_table.clone(), socket_table.clone());

    let service_table_clone = service_table.clone();
    let eventfd = nix::sys::eventfd::eventfd(0, nix::sys::eventfd::EfdFlags::EFD_CLOEXEC).unwrap();

    std::thread::spawn(move || {
        notification_handler::handle_all_streams(eventfd, service_table_clone);
    });
    
    // listen on signals from the child processes
    signal_handler::handle_signals(
        service_table.clone(),
        socket_table.clone(),
        pid_table.clone(),
        conf.notification_sockets_dir.clone(),
    );
}
