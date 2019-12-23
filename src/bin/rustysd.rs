#[macro_use]
extern crate log;
extern crate rustysd;

use rustysd::config;
use rustysd::control;
use rustysd::logging;
use rustysd::notification_handler;
use rustysd::platform;
use rustysd::signal_handler;
use rustysd::units;
use rustysd::wait_for_socket_activation;
use signal_hook::iterator::Signals;

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

    logging::setup_logging(&log_conf).unwrap();
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

    // TODO make configurable
    let should_go_to_new_session = false;
    if should_go_to_new_session {
        if !move_to_new_session() {
            return;
        }
    }

    rustysd::platform::become_subreaper(true);

    let signals = Signals::new(&[
        signal_hook::SIGCHLD,
        signal_hook::SIGTERM,
        signal_hook::SIGINT,
        signal_hook::SIGQUIT,
    ])
    .expect("Couldnt setup listening to the signals");

    // initial loading of the units and matching of the various before/after settings
    // also opening all fildescriptors in the socket files
    let unit_table = units::load_all_units(&conf.unit_dirs).unwrap();

    use std::sync::{Arc, Mutex, RwLock};
    //let service_table = Arc::new(Mutex::new(service_table));
    //let socket_table = Arc::new(Mutex::new(socket_table));
    let unit_table: std::collections::HashMap<_, _> = unit_table
        .into_iter()
        .map(|(id, unit)| (id, Arc::new(Mutex::new(unit))))
        .collect();
    let unit_table = Arc::new(RwLock::new(unit_table));
    let pid_table = Arc::new(Mutex::new(std::collections::HashMap::new()));

    // listen on user commands like listunits/kill/restart...
    let control_sock_path = conf.notification_sockets_dir.join("control.socket");
    if control_sock_path.exists() {
        std::fs::remove_file(&control_sock_path).unwrap();
    }

    // TODO make configurable
    use std::os::unix::net::UnixListener;
    std::fs::create_dir_all(&conf.notification_sockets_dir).unwrap();
    let unixsock = UnixListener::bind(&control_sock_path).unwrap();
    control::accept_control_connections_unix_socket(
        unit_table.clone(),
        pid_table.clone(),
        conf.notification_sockets_dir.clone(),
        unixsock,
    );
    let tcpsock = std::net::TcpListener::bind("127.0.0.1:8080").unwrap();
    control::accept_control_connections_tcp(
        unit_table.clone(),
        pid_table.clone(),
        conf.notification_sockets_dir.clone(),
        tcpsock,
    );

    let notification_eventfd = platform::make_event_fd().unwrap();
    let stdout_eventfd = platform::make_event_fd().unwrap();
    let stderr_eventfd = platform::make_event_fd().unwrap();
    let sock_act_eventfd = platform::make_event_fd().unwrap();
    let eventfds = vec![
        notification_eventfd,
        stdout_eventfd,
        stderr_eventfd,
        sock_act_eventfd,
    ];

    let unit_table_clone = unit_table.clone();
    std::thread::spawn(move || {
        notification_handler::handle_all_streams(notification_eventfd, unit_table_clone);
    });

    let unit_table_clone = unit_table.clone();
    std::thread::spawn(move || {
        notification_handler::handle_all_std_out(stdout_eventfd, unit_table_clone);
    });

    let unit_table_clone = unit_table.clone();
    std::thread::spawn(move || {
        notification_handler::handle_all_std_err(stderr_eventfd, unit_table_clone);
    });

    let unit_table_clone = unit_table.clone();
    let pid_table_clone = pid_table.clone();
    let note_sock_path_clone = conf.notification_sockets_dir.clone();
    let eventfds_clone = Arc::new(eventfds.clone());
    std::thread::spawn(move || loop {
        match wait_for_socket_activation::wait_for_socket(
            sock_act_eventfd,
            unit_table_clone.clone(),
        ) {
            Ok(ids) => {
                for socket_id in ids {
                    let unit_table_locked = unit_table_clone.read().unwrap();
                    {
                        let mut srvc_unit_id = None;
                        for unit in unit_table_locked.values() {
                            let unit_locked = unit.lock().unwrap();
                            if let crate::units::UnitSpecialized::Service(srvc) =
                                &unit_locked.specialized
                            {
                                if srvc.socket_ids.contains(&socket_id) {
                                    srvc_unit_id = Some(unit_locked.id);
                                }
                            }
                        }
                        if let Some(srvc_unit_id) = srvc_unit_id {
                            match crate::units::activate_unit::activate_unit(
                                srvc_unit_id,
                                None,
                                unit_table_clone.clone(),
                                pid_table_clone.clone(),
                                note_sock_path_clone.clone(),
                                eventfds_clone.clone(),
                                false,
                            ) {
                                Ok(_) => {
                                    /* TODO set all sockets to activated so they dont get listend to anymore here*/
                                    let srvc_unit = unit_table_locked.get(&srvc_unit_id).unwrap();
                                    let srvc_unit_locked = srvc_unit.lock().unwrap();
                                    if let crate::units::UnitSpecialized::Service(srvc) =
                                        &srvc_unit_locked.specialized
                                    {
                                        for sock_unit_id in &srvc.socket_ids {
                                            let sock_unit =
                                                unit_table_locked.get(&sock_unit_id).unwrap();
                                            let mut sock_unit_locked = sock_unit.lock().unwrap();
                                            if let crate::units::UnitSpecialized::Socket(sock) =
                                                &mut sock_unit_locked.specialized
                                            {
                                                sock.activated = true;
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    format!(
                                        "Error while starting service from socket activation: {}",
                                        e
                                    );
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                error!("Error in socket activation loop: {}", e);
                break;
            }
        }
    });

    // parallel startup of all services
    units::activate_unit::activate_units(
        unit_table.clone(),
        conf.notification_sockets_dir.clone(),
        eventfds.clone(),
        pid_table.clone(),
    );

    platform::notify_event_fds(&eventfds);

    // listen on signals from the child processes
    signal_handler::handle_signals(
        signals,
        unit_table,
        pid_table,
        conf.notification_sockets_dir.clone(),
        &eventfds,
    );
}
