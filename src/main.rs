use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use std::process::{Command, Stdio};

extern crate signal_hook;
use signal_hook::iterator::Signals;
extern crate libc;

extern crate nix;

mod unit_parser;

type InternalId = u64;

pub struct UnitConfig {
    wants: Vec<String>,
    wanted_by: Vec<String>,
    requires: Vec<String>,
    required_by: Vec<String>,
}

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
    id: InternalId,
    pid: Option<u32>,
    filepath: PathBuf,
    status: ServiceStatus,

    wants: Vec<InternalId>,
    requires: Vec<InternalId>,

    wanted_by: Vec<InternalId>,
    required_by: Vec<InternalId>,

    before: Vec<InternalId>,
    after: Vec<InternalId>,

    service_config: ServiceConfig,
    unit_config: UnitConfig,
}

fn run_services(services: &mut HashMap<InternalId, Service>, pids: &mut HashMap<u32, InternalId>) {
    for (_, srvc) in services {
        start_service(srvc);
        pids.insert(srvc.pid.unwrap(), srvc.id);
    }
}

fn start_service(srvc: &mut Service) {
    let split: Vec<&str> = srvc.service_config.exec.split(" ").collect();
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
        }
        Err(e) => panic!(e.description().to_owned()),
    }
}

fn fill_dependencies(services: &mut HashMap<InternalId, Service>) {
    let mut name_to_id = HashMap::new();

    for (id, srvc) in &*services {
        let name = srvc.filepath.file_name().unwrap().to_str().unwrap().to_owned();
        name_to_id.insert(name, *id);
    }

    let mut required_by = Vec::new();
    let mut wanted_by = Vec::new();

    for (_, srvc) in &mut *services {
        for name in &srvc.unit_config.wants {
            let id = name_to_id.get(name.as_str()).unwrap();
            srvc.wants.push(*id);
        }
        for name in &srvc.unit_config.requires {
            let id = name_to_id.get(name.as_str()).unwrap();
            srvc.requires.push(*id);
        }

        for name in &srvc.unit_config.wanted_by {
            let id = name_to_id.get(name.as_str()).unwrap();
            wanted_by.push((srvc.id, id));
        }
        for name in &srvc.unit_config.required_by {
            let id = name_to_id.get(name.as_str()).unwrap();
            required_by.push((srvc.id, id));
        }
    }

    for (wanted, wanting) in wanted_by {
        let srvc = services.get_mut(&wanting).unwrap();
        srvc.wants.push(wanted);
    }

    for (required, requiring) in required_by {
        let srvc = services.get_mut(&requiring).unwrap();
        srvc.requires.push(required);
    }
}

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

    fill_dependencies(&mut service_table);


    let mut pid_table = HashMap::new();
    run_services(&mut service_table, &mut pid_table);

    loop {
        // Pick up new signals
        for signal in signals.forever() {
            match signal as libc::c_int {
                signal_hook::SIGCHLD => {
                    match nix::sys::wait::waitpid(-1, Some(nix::sys::wait::WNOHANG)) {
                        Ok(exit_status) => match exit_status {
                            nix::sys::wait::WaitStatus::Exited(pid, code) => {
                                let srvc_id = pid_table.get(&(pid as u32)).unwrap();
                                let srvc = service_table.get_mut(&srvc_id).unwrap();
                                println!(
                                    "Service with id: {} pid: {} exited with code: {}",
                                    srvc_id, pid, code
                                );

                                pid_table.remove(&(pid as u32));
                                srvc.status = ServiceStatus::Stopped;

                                if srvc.service_config.keep_alive {
                                    start_service(srvc);
                                    pid_table.insert(srvc.pid.unwrap(), srvc.id);
                                }
                            }
                            _ => {
                                println!("Child exited with code: {:?}", exit_status);
                            }
                        },
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
