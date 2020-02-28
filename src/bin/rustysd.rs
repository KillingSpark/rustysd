#[macro_use]
extern crate log;
extern crate rustysd;

use rustysd::config;
use rustysd::control;
use rustysd::logging;
use rustysd::notification_handler;
use rustysd::platform;
use rustysd::signal_handler;
use rustysd::socket_activation;
use rustysd::units;
use signal_hook::iterator::Signals;
use std::sync::{Arc, Mutex, RwLock};

fn find_shell_path() -> Option<std::path::PathBuf> {
    let possible_paths = vec![
        std::path::PathBuf::from("/bin/sh"),
        std::path::PathBuf::from("/sbin/sh"),
        std::path::PathBuf::from("/usr/bin/sh"),
    ];

    // TODO make configurable
    for p in possible_paths {
        if p.exists() {
            return Some(p);
        }
    }
    None
}

fn unrecoverable_error(error: String) {
    if nix::unistd::getpid().as_raw() == 1 {
        eprintln!("Unrecoverable error: {}", error);
        if let Some(shell_path) = find_shell_path() {
            match std::process::Command::new(shell_path).spawn() {
                Ok(mut child) => match child.wait() {
                    Ok(_) => {
                        let dur = std::time::Duration::from_secs(10);
                        eprintln!("Returned from shell. Will exit after sleeping: {:?}", dur);
                        std::thread::sleep(dur);
                        std::process::exit(1);
                    }
                    Err(e) => {
                        let dur = std::time::Duration::from_secs(1_000_000);
                        eprintln!(
                            "Error while waiting on the shell: {}. Will sleep for {:?} and then exit",
                            e, dur
                        );
                        std::thread::sleep(dur);
                        std::process::exit(1);
                    }
                },
                Err(e) => {
                    let dur = std::time::Duration::from_secs(1_000_000);
                    eprintln!(
                        "Error while starting the shell: {}. Will sleep for {:?} and then exit",
                        e, dur
                    );
                    std::thread::sleep(dur);
                    std::process::exit(1);
                }
            }
        } else {
            let dur = std::time::Duration::from_secs(10);
            eprintln!(
                "Cannot find a shell for emergency. Will sleep for {:?} and then exit",
                dur
            );
            std::thread::sleep(dur);
            std::process::exit(1);
        }
    } else {
        panic!("{}", error);
    }
}

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

#[cfg(target_os = "linux")]
fn remount_root_rw() {
    // TODO maybe need more flags
    let flags = nix::mount::MsFlags::MS_REMOUNT;
    let source: Option<&str> = None;
    let fs_type: Option<&str> = None;
    let data: Option<&str> = None;
    nix::mount::mount(source, "/", fs_type, flags, data).unwrap();
}

#[cfg(target_os = "linux")]
fn pid1_specific_setup() {
    if nix::unistd::getpid().as_raw() == 0 {
        remount_root_rw();
    }
}
#[cfg(not(target_os = "linux"))]
fn pid1_specific_setup() {}

fn prepare_runtimeinfo(conf: &config::Config, dry_run: bool) -> Arc<units::RuntimeInfo> {
    // initial loading of the units and matching of the various before/after settings
    // also opening all fildescriptors in the socket files
    let mut first_id = 0;
    let unit_table =
        units::load_all_units(&conf.unit_dirs, &mut first_id, &conf.target_unit).unwrap();
    trace!("Finished loading units");
    first_id = first_id + 1;

    if let Err(e) = units::sanity_check_dependencies(&unit_table) {
        match e {
            units::SanityCheckError::CirclesFound(circles) => {
                error!("Found {} cycle(s) in the dependencies", circles.len());
                for circle in &circles {
                    error!("-- Next circle --");
                    for id in circle {
                        error!("{}", id);
                    }
                    error!("-- End circle --");
                }
            }
            units::SanityCheckError::Generic(msg) => {
                error!("Unit dependencies did not pass sanity checks: {}", msg);
            }
        }
        unrecoverable_error("Unit dependencies did not pass sanity check".into());
    }
    trace!("Unit dependencies passed sanity checks");
    let unit_table = unit_table;

    if dry_run {
        warn!("Exit after loading because --dry-run was passed");
        unrecoverable_error("Started as dry-run".into());
    }

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

        last_id: Arc::new(Mutex::new(first_id)),
        config: conf.clone(),
    });

    run_info
}

fn start_notification_handler_thread(run_info: units::ArcRuntimeInfo, eventfd: platform::EventFd) {
    std::thread::spawn(move || {
        notification_handler::handle_all_streams(eventfd, run_info.unit_table.clone());
    });
}
fn start_stdout_handler_thread(run_info: units::ArcRuntimeInfo, eventfd: platform::EventFd) {
    std::thread::spawn(move || {
        notification_handler::handle_all_std_out(eventfd, run_info.clone());
    });
}
fn start_stderr_handler_thread(run_info: units::ArcRuntimeInfo, eventfd: platform::EventFd) {
    std::thread::spawn(move || {
        notification_handler::handle_all_std_err(eventfd, run_info.clone());
    });
}
fn start_signal_handler_thread(
    signals: Signals,
    run_info: units::ArcRuntimeInfo,
    conf: &config::Config,
    eventfds: Vec<platform::EventFd>,
) -> std::thread::JoinHandle<()> {
    let note_conf_dir = conf.notification_sockets_dir.clone();
    let handle = std::thread::spawn(move || {
        // listen on signals from the child processes
        signal_handler::handle_signals(signals, run_info, note_conf_dir, eventfds);
    });
    handle
}

#[derive(Default)]
struct CliArgs {
    conf_path: Option<std::path::PathBuf>,
    dry_run: bool,
    show_help: bool,
    unknown_arg: Option<String>
}

fn parse_args() -> CliArgs {
    let args = std::env::args().collect::<Vec<_>>();
    // ignore exec name
    let args = &args[1..];

    let mut cli_args = CliArgs::default();
    let mut idx = 0;
    while idx < args.len() {
        match args[idx].as_str() {
            "-c" | "--config" => {
                if args.len() < idx {
                    unrecoverable_error(format!("config flag set but no path given"));
                } else {
                    let path_str = args[idx + 1].clone();
                    let p = std::path::PathBuf::from(path_str);
                    if !p.exists() {
                        unrecoverable_error(format!("config path given that does not exist"));
                    }
                    if !p.is_dir() {
                        unrecoverable_error(format!("config path given that is not a directory"));
                    }
                    cli_args.conf_path = Some(p);
                    idx += 2;
                }
            }
            "-d" | "--dry-run" => {
                cli_args.dry_run = true;
                idx += 1;
            }
            "-h" | "--help" => {
                cli_args.show_help = true;
                idx += 1;
            }
            unknown => {
                cli_args.unknown_arg = Some(unknown.to_string());
                break;
            }
        }
    }
    cli_args
}

fn main() {
    pid1_specific_setup();

    let cli_args = parse_args();

    let usage = "Usage: rustysd [-c | --config PATH] [-d | --dry-run] [-h | --help]";
    if cli_args.show_help {
        println!("{}", usage);
        std::process::exit(0);
    } else if let Some(unknown) = cli_args.unknown_arg {
        unrecoverable_error(format!("{}\n\nUnknown cli arg: {}", usage, unknown));
    }

    let (log_conf, conf) = config::load_config(&cli_args.conf_path);

    logging::setup_logging(&log_conf).unwrap();
    let conf = match conf {
        Ok(conf) => conf,
        Err(e) => {
            error!("Error while loading the conf: {}", e);
            unrecoverable_error(format!(
                "Reading conf did not work. See stdout or log at: {:?}",
                log_conf.log_dir
            ));
            // unrecoverable_error always shutsdown rustysd
            unreachable!("");
        }
    };

    #[cfg(feature = "cgroups")]
    {
        platform::cgroups::move_to_own_cgroup(&std::path::PathBuf::from("/sys/fs/cgroup")).unwrap();
    }

    // TODO make configurable
    let should_go_to_new_session = false;
    if should_go_to_new_session {
        if !move_to_new_session() {
            return;
        }
    }

    rustysd::platform::become_subreaper(true);

    let run_info = prepare_runtimeinfo(&conf, cli_args.dry_run);

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

    let signals = match Signals::new(&[
        signal_hook::SIGCHLD,
        signal_hook::SIGTERM,
        signal_hook::SIGINT,
        signal_hook::SIGQUIT,
    ]) {
        Ok(signals) => signals,
        Err(e) => {
            unrecoverable_error(format!("Couldnt setup listening to the signals: {}", e));
            // unrecoverable_error always shutsdown rustysd
            unreachable!("");
        }
    };
    // listen to signals
    let handle = start_signal_handler_thread(signals, run_info.clone(), &conf, eventfds.clone());

    // listen on user commands like listunits/kill/restart...
    control::open_all_sockets(run_info.clone(), &conf);

    start_notification_handler_thread(run_info.clone(), notification_eventfd);
    start_stdout_handler_thread(run_info.clone(), stdout_eventfd);
    start_stderr_handler_thread(run_info.clone(), stderr_eventfd);

    socket_activation::start_socketactivation_thread(
        run_info.clone(),
        conf.notification_sockets_dir.clone(),
        sock_act_eventfd,
        Arc::new(eventfds.clone()),
    );

    // parallel startup of all services
    units::activate_units(
        run_info.clone(),
        conf.notification_sockets_dir.clone(),
        eventfds.clone(),
    );

    handle.join().unwrap();
}
