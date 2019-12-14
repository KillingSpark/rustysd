mod config;
mod control;
mod dbus_wait;
mod logging;
mod notification_handler;
mod services;
mod signal_handler;
mod sockets;
mod unit_parser;
mod units;

extern crate signal_hook;

#[macro_use]
extern crate log;
extern crate dbus;
extern crate fern;
extern crate lumberjack_rs;
extern crate serde_json;
extern crate threadpool;

fn move_to_new_session() -> bool {
    match nix::unistd::fork() {
        Ok(nix::unistd::ForkResult::Child) => {
            nix::unistd::setsid().unwrap();
            true
        }
        Ok(nix::unistd::ForkResult::Parent { .. }) => false,
        Err(e) => {
            error!("Fork before setsid failed: {}", e);
            false
        }
    }
}

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

    let should_go_to_new_session = false;
    if should_go_to_new_session {
        if !move_to_new_session() {
            return;
        }
    }

    // initial loading of the units and matching of the various before/after settings
    // also opening all fildescriptors in the socket files
    let (service_table, socket_table) = unit_parser::load_all_units(&conf.unit_dirs).unwrap();

    use std::sync::{Arc, Mutex};
    let service_table = Arc::new(Mutex::new(service_table));
    let socket_table = Arc::new(Mutex::new(socket_table));

    // listen on user commands like listunits/kill/restart...
    control::accept_control_connections(service_table.clone(), socket_table.clone());

    let notification_eventfd =
        nix::sys::eventfd::eventfd(0, nix::sys::eventfd::EfdFlags::EFD_CLOEXEC).unwrap();
    let stdout_eventfd =
        nix::sys::eventfd::eventfd(0, nix::sys::eventfd::EfdFlags::EFD_CLOEXEC).unwrap();
    let stderr_eventfd =
        nix::sys::eventfd::eventfd(0, nix::sys::eventfd::EfdFlags::EFD_CLOEXEC).unwrap();

    let service_table_clone = service_table.clone();
    let service_table_clone2 = service_table.clone();
    let service_table_clone3 = service_table.clone();

    std::thread::spawn(move || {
        notification_handler::handle_all_streams(notification_eventfd, service_table_clone);
    });

    std::thread::spawn(move || {
        notification_handler::handle_all_std_out(stdout_eventfd, service_table_clone2);
    });
    std::thread::spawn(move || {
        notification_handler::handle_all_std_err(stderr_eventfd, service_table_clone3);
    });

    let eventfds = vec![notification_eventfd, stdout_eventfd, stderr_eventfd];

    // parallel startup of all services
    let pid_table = services::run_services(
        service_table.clone(),
        socket_table.clone(),
        conf.notification_sockets_dir.clone(),
        eventfds.clone(),
    );

    notification_handler::notify_event_fds(&eventfds);

    // listen on signals from the child processes
    signal_handler::handle_signals(
        service_table.clone(),
        socket_table.clone(),
        pid_table.clone(),
        conf.notification_sockets_dir.clone(),
    );
}
