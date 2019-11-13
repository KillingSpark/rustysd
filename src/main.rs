mod services;
mod unit_parser;

extern crate signal_hook;
use signal_hook::iterator::Signals;

use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;

fn main() {
    let signals =
        Signals::new(&[signal_hook::SIGCHLD]).expect("Couldnt setup listening to the signals");

    let mut service_table = HashMap::new();
    let mut base_id = 0;
    unit_parser::parse_all_services(
        &mut service_table,
        &PathBuf::from("./test_units"),
        &mut base_id,
    );

    let name_to_id = services::fill_dependencies(&mut service_table);
    for (_, srvc) in &mut service_table {
        srvc.dedup_dependencies();
    }

    services::print_all_services(&service_table);

    let mut pid_table = HashMap::new();
    services::run_services(&mut service_table, &name_to_id, &mut pid_table);

    loop {
        // Pick up new signals
        for signal in signals.forever() {
            match signal as libc::c_int {
                signal_hook::SIGCHLD => {
                    for (pid, code) in std::iter::from_fn(get_next_exited_child) {
                        services::service_exit_handler(
                            pid,
                            code,
                            &mut service_table,
                            &mut pid_table,
                        )
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}

fn get_next_exited_child() -> Option<(i32, i8)> {
    match nix::sys::wait::waitpid(-1, Some(nix::sys::wait::WNOHANG)) {
        Ok(exit_status) => match exit_status {
            nix::sys::wait::WaitStatus::Exited(pid, code) => Some((pid, code)),
            nix::sys::wait::WaitStatus::Signaled(pid, signal, dumped_core) => {
                // signals get handed to the parent if the child got killed by it but didnt handle the
                // signal itself
                if signal == libc::SIGTERM {
                    if dumped_core {
                        Some((pid, signal as i8))
                    } else {
                        Some((pid, signal as i8))
                    }
                } else {
                    None
                }
            }
            nix::sys::wait::WaitStatus::StillAlive => {
                println!("No more state changes to poll");
                None
            }
            _ => {
                println!("Child signaled with code: {:?}", exit_status);
                None
            }
        },
        Err(e) => {
            if let nix::Error::Sys(nix::errno::ECHILD) = e {
            } else {
                println!("Error while waiting: {}", e.description().to_owned());
            }
            None
        }
    }
}
