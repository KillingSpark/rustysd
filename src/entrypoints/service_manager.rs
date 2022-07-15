use log::{error, trace, warn};
use signal_hook::iterator::Signals;
use std::sync::{Arc, Mutex, RwLock};

use crate::config;
use crate::control;
use crate::logging;
use crate::notification_handler;
use crate::platform;
use crate::runtime_info;
use crate::signal_handler;
use crate::socket_activation;
use crate::units;

pub fn run_service_manager() {
    pid1_specific_setup();

    let cli_args = CliArgs::try_parse().unwrap_or_else(|e| {
        unrecoverable_error(e.to_string());
        unreachable!();
    });

    if let Some(path) = &cli_args.conf {
        if !path.exists() {
            unrecoverable_error(format!("config path given that does not exist"));
        }
        if !path.is_dir() {
            unrecoverable_error(format!("config path given that is not a directory"));
        }
    }

    let (log_conf, conf) = config::load_config(&cli_args.conf);

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

    crate::platform::become_subreaper(true);

    let run_info = prepare_runtimeinfo(&conf, cli_args.dry_run);

    let signals = match Signals::new(&[
        signal_hook::consts::SIGCHLD,
        signal_hook::consts::SIGTERM,
        signal_hook::consts::SIGINT,
        signal_hook::consts::SIGQUIT,
    ]) {
        Ok(signals) => signals,
        Err(e) => {
            unrecoverable_error(format!("Couldnt setup listening to the signals: {}", e));
            // unrecoverable_error always shutsdown rustysd
            unreachable!("");
        }
    };
    // listen to signals
    let handle = start_signal_handler_thread(signals, run_info.clone());

    // listen on user commands like listunits/kill/restart...
    control::open_all_sockets(run_info.clone(), &conf);

    start_notification_handler_thread(run_info.clone());
    start_stdout_handler_thread(run_info.clone());
    start_stderr_handler_thread(run_info.clone());

    socket_activation::start_socketactivation_thread(run_info.clone());

    trace!("Started all helper threads. Start activating units");

    let target_id: units::UnitId = {
        let run_info: &runtime_info::RuntimeInfo = &*run_info.read().unwrap();
        use std::convert::TryInto;
        run_info.config.target_unit.as_str().try_into().unwrap()
    };

    // parallel startup of all services
    units::activate_needed_units(target_id, run_info.clone());

    handle.join().unwrap();
}

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
    match unsafe { nix::unistd::fork() } {
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

fn prepare_runtimeinfo(conf: &config::Config, dry_run: bool) -> runtime_info::ArcMutRuntimeInfo {
    // initial loading of the units and matching of the various before/after settings
    // also opening all fildescriptors in the socket files
    let unit_table =
        units::load_all_units(&conf.unit_dirs, &conf.target_unit).expect("loading unit files");
    trace!("Finished loading units");
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

    let pid_table = Mutex::new(std::collections::HashMap::new());

    let run_info = Arc::new(RwLock::new(runtime_info::RuntimeInfo {
        unit_table: unit_table,
        pid_table: pid_table,
        fd_store: std::sync::RwLock::new(crate::fd_store::FDStore::default()),
        config: conf.clone(),
        stdout_eventfd: platform::make_event_fd().unwrap(),
        stderr_eventfd: platform::make_event_fd().unwrap(),
        notification_eventfd: platform::make_event_fd().unwrap(),
        socket_activation_eventfd: platform::make_event_fd().unwrap(),
    }));

    run_info
}

fn start_notification_handler_thread(run_info: runtime_info::ArcMutRuntimeInfo) {
    std::thread::spawn(move || {
        notification_handler::handle_all_streams(run_info.clone());
    });
}
fn start_stdout_handler_thread(run_info: runtime_info::ArcMutRuntimeInfo) {
    std::thread::spawn(move || {
        notification_handler::handle_all_std_out(run_info.clone());
    });
}
fn start_stderr_handler_thread(run_info: runtime_info::ArcMutRuntimeInfo) {
    std::thread::spawn(move || {
        notification_handler::handle_all_std_err(run_info.clone());
    });
}
fn start_signal_handler_thread(
    signals: Signals,
    run_info: runtime_info::ArcMutRuntimeInfo,
) -> std::thread::JoinHandle<()> {
    let handle = std::thread::spawn(move || {
        // listen on signals from the child processes
        signal_handler::handle_signals(signals, run_info);
    });
    handle
}

use clap::Parser;

#[derive(Parser, Debug)]
struct CliArgs {
    #[clap(short, long, value_parser)]
    conf: Option<std::path::PathBuf>,
    #[clap(short, long, value_parser)]
    dry_run: bool,
}
