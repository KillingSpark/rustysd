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
    let mut unit_table = units::load_all_units(&conf.unit_dirs).unwrap();
    units::prune_units(&conf.target_unit, &mut unit_table).unwrap();
    units::sanity_check_dependencies(&unit_table).unwrap();
    trace!("Unit dependencies passed sanity checks");
    let unit_table = unit_table;

    if std::env::args()
        .collect::<Vec<_>>()
        .contains(&"--dry-run".to_owned())
    {
        warn!("Exit after loading because --dry-run was passed");
        return;
    }

    use std::sync::{Arc, Mutex, RwLock};
    // wrap units into mutexes
    let unit_table: std::collections::HashMap<_, _> = unit_table
        .into_iter()
        .map(|(id, unit)| (id, Arc::new(Mutex::new(unit))))
        .collect();
    let unit_table = Arc::new(RwLock::new(unit_table));

    // init the status map
    let mut status_table = std::collections::HashMap::new();
    for id in unit_table.read().unwrap().keys() {
        status_table.insert(*id, Arc::new(Mutex::new(units::UnitStatus::NeverStarted)));
    }
    let status_table = Arc::new(RwLock::new(status_table));

    let pid_table = Arc::new(Mutex::new(std::collections::HashMap::new()));

    let run_info = Arc::new(units::RuntimeInfo {
        unit_table: unit_table.clone(),
        pid_table: pid_table.clone(),
        fd_store: Arc::new(std::sync::RwLock::new(rustysd::fd_store::FDStore::default())),
        status_table: status_table.clone(),

        // TODO find actual max id used
        last_id: Arc::new(Mutex::new(300)),
        config: conf.clone(),
    });

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
        run_info.clone(),
        conf.notification_sockets_dir.clone(),
        unixsock,
    );
    let tcpsock = std::net::TcpListener::bind("127.0.0.1:8080").unwrap();
    control::accept_control_connections_tcp(
        run_info.clone(),
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

    let run_info_clone = run_info.clone();
    let note_sock_path_clone = conf.notification_sockets_dir.clone();
    let eventfds_clone = Arc::new(eventfds.clone());
    std::thread::spawn(move || loop {
        match wait_for_socket_activation::wait_for_socket(
            sock_act_eventfd,
            run_info_clone.unit_table.clone(),
            run_info_clone.fd_store.clone(),
        ) {
            Ok(ids) => {
                for socket_id in ids {
                    let unit_table_locked = run_info_clone.unit_table.read().unwrap();
                    {
                        let socket_name = {
                            let sock_unit = unit_table_locked.get(&socket_id).unwrap();
                            let sock_unit_locked = sock_unit.lock().unwrap();
                            sock_unit_locked.conf.name()
                        };

                        let mut srvc_unit_id = None;
                        for unit in unit_table_locked.values() {
                            let unit_locked = unit.lock().unwrap();
                            if let crate::units::UnitSpecialized::Service(srvc) =
                                &unit_locked.specialized
                            {
                                if srvc.socket_names.contains(&socket_name) {
                                    srvc_unit_id = Some(unit_locked.id);
                                    trace!(
                                        "Start service {} by socket activation",
                                        unit_locked.conf.name()
                                    );
                                }
                            }
                        }

                        if let Some(srvc_unit_id) = srvc_unit_id {
                            if let Some(status) = run_info_clone
                                .status_table
                                .read()
                                .unwrap()
                                .get(&srvc_unit_id)
                            {
                                let srvc_status = {
                                    let status_locked = status.lock().unwrap();
                                    *status_locked
                                };
                                
                                if srvc_status != units::UnitStatus::StartedWaitingForSocket {
                                    trace!(
                                        "Ignore socket activation. Service has status: {:?}",
                                        srvc_status
                                    );
                                    let sock_unit = unit_table_locked.get(&socket_id).unwrap();
                                    let mut sock_unit_locked = sock_unit.lock().unwrap();
                                    if let units::UnitSpecialized::Socket(sock) =
                                        &mut sock_unit_locked.specialized
                                    {
                                        sock.activated = true;
                                    }
                                } else {
                                    match crate::units::activate_unit(
                                        srvc_unit_id,
                                        run_info_clone.clone(),
                                        note_sock_path_clone.clone(),
                                        eventfds_clone.clone(),
                                        false,
                                    ) {
                                        Ok(_) => {
                                            let sock_unit =
                                                unit_table_locked.get(&socket_id).unwrap();
                                            let mut sock_unit_locked = sock_unit.lock().unwrap();
                                            if let units::UnitSpecialized::Socket(sock) =
                                                &mut sock_unit_locked.specialized
                                            {
                                                sock.activated = true;
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
                }
            }
            Err(e) => {
                error!("Error in socket activation loop: {}", e);
                break;
            }
        }
    });

    platform::notify_event_fds(&eventfds);

    // listen to signals
    let run_info_clone = run_info.clone();
    let note_dir_clone = conf.notification_sockets_dir.clone();
    let eventfds_clone = eventfds.clone();
    let handle = std::thread::spawn(move || {
        // listen on signals from the child processes
        signal_handler::handle_signals(signals, run_info_clone, note_dir_clone, eventfds_clone);
    });

    // parallel startup of all services
    units::activate_units(
        run_info.clone(),
        conf.notification_sockets_dir.clone(),
        eventfds.clone(),
    );

    handle.join().unwrap();
}
