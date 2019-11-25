mod services;
mod sockets;
mod unit_parser;
mod units;
use units::*;

extern crate signal_hook;
use signal_hook::iterator::Signals;

#[macro_use]
extern crate log;
extern crate crossbeam;
extern crate fern;
extern crate lumberjack_rs;
extern crate threadpool;

use std::collections::HashMap;
use std::error::Error;
use std::io::Write;
use std::path::PathBuf;

fn main() {
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
        .apply()
        .unwrap();

    let signals =
        Signals::new(&[signal_hook::SIGCHLD]).expect("Couldnt setup listening to the signals");

    let mut service_table = HashMap::new();
    let mut base_id = 0;
    unit_parser::parse_all_services(
        &mut service_table,
        &PathBuf::from("./test_units"),
        &mut base_id,
    );

    let mut socket_unit_table = HashMap::new();
    unit_parser::parse_all_sockets(
        &mut socket_unit_table,
        &PathBuf::from("./test_units"),
        &mut base_id,
    );

    let _name_to_id = units::fill_dependencies(&mut service_table);
    for (_, srvc) in &mut service_table {
        srvc.dedup_dependencies();
    }

    let service_table = apply_sockets_to_services(service_table, &socket_unit_table);

    let mut socket_table = HashMap::new();
    for (_id, socket_unit) in socket_unit_table {
        if let UnitSpecialized::Socket(socket) = socket_unit.specialized {
            socket_table.insert(socket.name.clone(), socket);
        }
    }

    sockets::open_all_sockets(&mut socket_table).unwrap();

    services::print_all_services(&service_table);

    let pid_table = HashMap::new();
    let (mut service_table, mut pid_table) =
        services::run_services(service_table, pid_table, socket_table.clone());

    loop {
        // Pick up new signals
        for signal in signals.forever() {
            match signal as libc::c_int {
                signal_hook::SIGCHLD => {
                    std::iter::from_fn(get_next_exited_child)
                        .take_while(Result::is_ok)
                        .for_each(|val| match val {
                            Ok((pid, code)) => services::service_exit_handler(
                                pid,
                                code,
                                &mut service_table,
                                &mut pid_table,
                                &socket_table,
                            ),
                            Err(e) => {
                                error!("{}", e);
                            }
                        });
                }

                _ => unreachable!(),
            }
        }
    }
}

fn apply_sockets_to_services(
    mut service_table: HashMap<InternalId, Unit>,
    socket_table: &HashMap<InternalId, Unit>,
) -> HashMap<InternalId, Unit> {
    for (_, sock_unit) in socket_table {
        if let UnitSpecialized::Socket(sock) = &sock_unit.specialized {
            trace!("Searching service for socket: {}", sock_unit.conf.name());
            for (_, srvc_unit) in &mut service_table {
                let srvc = &mut srvc_unit.specialized;
                if let UnitSpecialized::Service(srvc) = srvc {
                    if srvc_unit.conf.name() == sock_unit.conf.name() {
                        trace!(
                            "add socket: {} to service: {}",
                            sock_unit.conf.name(),
                            srvc_unit.conf.name()
                        );

                        srvc.sockets.push(sock.name.clone());
                    }
                }
            }
        }
    }

    service_table
}

fn get_next_exited_child() -> Option<Result<(i32, i8), nix::Error>> {
    match nix::sys::wait::waitpid(-1, Some(nix::sys::wait::WNOHANG)) {
        Ok(exit_status) => match exit_status {
            nix::sys::wait::WaitStatus::Exited(pid, code) => Some(Ok((pid, code))),
            nix::sys::wait::WaitStatus::Signaled(pid, signal, dumped_core) => {
                // signals get handed to the parent if the child got killed by it but didnt handle the
                // signal itself
                if signal == libc::SIGTERM {
                    if dumped_core {
                        Some(Ok((pid, signal as i8)))
                    } else {
                        Some(Ok((pid, signal as i8)))
                    }
                } else {
                    None
                }
            }
            nix::sys::wait::WaitStatus::StillAlive => {
                trace!("No more state changes to poll");
                None
            }
            _ => {
                trace!("Child signaled with code: {:?}", exit_status);
                None
            }
        },
        Err(e) => {
            if let nix::Error::Sys(nix::errno::ECHILD) = e {
            } else {
                trace!("Error while waiting: {}", e.description().to_owned());
            }
            Some(Err(e))
        }
    }
}
