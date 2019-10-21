use std::error::Error;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::collections::HashMap;

extern crate signal_hook;
use signal_hook::iterator::Signals;
extern crate libc;

extern crate nix;

mod unit_parser;

type internalId = u64;

pub struct ServiceConfig {
    keep_alive: bool,
    exec: String,
    stop: String,
}

pub enum ServiceStatus {
    NeverRan,
    Running,
    Stopped,
}

pub struct Service {
    id: internalId,
    pid: Option<u32>,
    filepath: PathBuf,
    status: ServiceStatus,

    wants: Vec<internalId>,
    requires: Vec<internalId>,

    wanted_by: Vec<internalId>,
    required_by: Vec<internalId>,

    before: Vec<internalId>,
    after: Vec<internalId>,

    config: ServiceConfig,
}

fn run_services(services: &mut HashMap<internalId, Service>, pids: &mut HashMap<u32, internalId>) {
    for (_, srvc) in services {
        start_service(srvc);
        pids.insert(srvc.pid.unwrap(), srvc.id);
    }
}


fn start_service(srvc: &mut Service) {
    let split: Vec<&str> = srvc.config.exec.split(" ").collect();
        let mut cmd = Command::new(split[0]);
        for part in &split[1..] {
            cmd.arg(part);
        }
        cmd.stdout(Stdio::null());

        match cmd.spawn() {
            Ok(child) => {
                srvc.pid = Some(child.id());
                srvc.status = ServiceStatus::Running;
                
                println!("started: {}", srvc.pid.unwrap());
                },
            Err(e) => panic!(e.description().to_owned()),
        }
}

fn main() {
    let signals = Signals::new(&[
        signal_hook::SIGCHLD,
    ]).expect("Couldnt setup listening to the signals");

    let mut service_table = HashMap::new();
    let mut pid_table = HashMap::new();
    let mut base_id = 0;
    unit_parser::parse_all_services(&mut service_table, &PathBuf::from("./test_units"), &mut base_id);

    run_services(&mut service_table, &mut pid_table);

    loop {
        // Pick up new signals
        for signal in signals.forever() {
            match signal as libc::c_int {
                signal_hook::SIGCHLD => {
                    match nix::sys::wait::waitpid(-1, Some(nix::sys::wait::WNOHANG)) {
                        Ok(exit_status) => {
                            match exit_status {
                                nix::sys::wait::WaitStatus::Exited(pid, code) => {
                                    let srvc_id = pid_table.get(&(pid as u32)).unwrap();
                                    let srvc = service_table.get_mut(&srvc_id).unwrap();
                                    println!("Service with id: {} pid: {} exited with code: {}", srvc_id, pid, code);

                                    pid_table.remove(&(pid as u32));
                                    srvc.status = ServiceStatus::Stopped;

                                    if srvc.config.keep_alive {
                                        start_service(srvc);
                                        pid_table.insert(srvc.pid.unwrap(), srvc.id);
                                    }
                                }
                                _ => {
                                    println!("Child exited with code: {:?}", exit_status);
                                }
                            }
                        }
                        Err(e) => {
                            println!("Error while waiting: {}", e.description().to_owned());
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}
