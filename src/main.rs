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
    requires: Vec<String>,
    before: Vec<String>,
    after: Vec<String>,
}
pub struct InstallConfig {
    wanted_by: Vec<String>,
    required_by: Vec<String>,
}

pub struct ServiceConfig {
    keep_alive: bool,
    exec: String,
    stop: String,
}

pub enum ServiceStatus {
    NeverRan,
    Starting,
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

    service_config: Option<ServiceConfig>,
    unit_config: Option<UnitConfig>,
    install_config: Option<InstallConfig>,
}

impl Service {
    fn name(&self) -> String {
        let name = self
            .filepath
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
        let name = name.trim_end_matches(".service").to_owned();

        name
    }
}

fn run_services_recursive(
    ids_to_start: Vec<InternalId>,
    services: &mut HashMap<InternalId, Service>,
    name_to_id: &HashMap<String, InternalId>,
    pids: &mut HashMap<u32, InternalId>,
) {
    for id in ids_to_start {
        let srvc = services.get_mut(&id).unwrap();
        match srvc.status {
            ServiceStatus::NeverRan => {
                start_service(srvc);
                pids.insert(srvc.pid.unwrap(), srvc.id);
            }
            _ => { /*ignore. Was run by another tree*/ }
        }

        run_services_recursive(srvc.before.clone(), services, name_to_id, pids);
    }
}

fn run_services(
    services: &mut HashMap<InternalId, Service>,
    name_to_id: &HashMap<String, InternalId>,
    pids: &mut HashMap<u32, InternalId>,
) {
    let mut root_services = Vec::new();

    for (id, srvc) in &*services {
        if srvc.after.len() == 0 {
            root_services.push(*id);
        }
    }

    run_services_recursive(root_services, services, name_to_id, pids);
}

fn start_service(srvc: &mut Service) {
    srvc.status = ServiceStatus::Starting;

    let split: Vec<&str> = match &srvc.service_config {
        Some(conf) => conf.exec.split(" ").collect(),
        None => return,
    };

    let mut cmd = Command::new(split[0]);
    for part in &split[1..] {
        cmd.arg(part);
    }
    cmd.stdout(Stdio::null());

    match cmd.spawn() {
        Ok(child) => {
            srvc.pid = Some(child.id());
            srvc.status = ServiceStatus::Running;

            println!(
                "Service: {} started with pid: {}",
                srvc.name(),
                srvc.pid.unwrap()
            );
        }
        Err(e) => panic!(e.description().to_owned()),
    }
}

fn print_all_services(services: &HashMap<InternalId, Service>) {
    for (id, srvc) in services {
        println!("{}:", id);
        println!("  {}", srvc.name());
        println!("  Before {:?}", srvc.before);
        println!("  After {:?}", srvc.after);
    }
}

fn fill_dependencies(services: &mut HashMap<InternalId, Service>) -> HashMap<String, u64> {
    let mut name_to_id = HashMap::new();

    for (id, srvc) in &*services {
        let name = srvc.name();
        name_to_id.insert(name, *id);
    }

    let mut required_by = Vec::new();
    let mut wanted_by = Vec::new();
    let mut before = Vec::new();
    let mut after = Vec::new();

    for (_, srvc) in &mut *services {
        if let Some(conf) = &srvc.unit_config {
            for name in &conf.wants {
                let id = name_to_id.get(name.as_str()).unwrap();
                srvc.wants.push(*id);
            }
            for name in &conf.requires {
                let id = name_to_id.get(name.as_str()).unwrap();
                srvc.requires.push(*id);
            }
            for name in &conf.before {
                let id = name_to_id.get(name.as_str()).unwrap();
                srvc.before.push(*id);
                after.push((srvc.id, *id))
            }
            for name in &conf.after {
                let id = name_to_id.get(name.as_str()).unwrap();
                srvc.after.push(*id);
                before.push((srvc.id, *id))
            }
        }

        if let Some(conf) = &srvc.install_config {
            for name in &conf.wanted_by {
                let id = name_to_id.get(name.as_str()).unwrap();
                wanted_by.push((srvc.id, id));
            }
            for name in &conf.required_by {
                let id = name_to_id.get(name.as_str()).unwrap();
                required_by.push((srvc.id, id));
            }
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

    for (before, after) in before {
        let srvc = services.get_mut(&after).unwrap();
        srvc.before.push(before);
    }
    for (after, before) in after {
        let srvc = services.get_mut(&before).unwrap();
        srvc.after.push(after);
    }

    name_to_id
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

    let name_to_id = fill_dependencies(&mut service_table);

    print_all_services(&service_table);

    let mut pid_table = HashMap::new();
    run_services(&mut service_table, &name_to_id, &mut pid_table);

    loop {
        // Pick up new signals
        for signal in signals.forever() {
            match signal as libc::c_int {
                signal_hook::SIGCHLD => {
                    let mut more_pids = true;
                    while more_pids {
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

                                    if let Some(conf) = &srvc.service_config {
                                        if conf.keep_alive {
                                            start_service(srvc);
                                            pid_table.insert(srvc.pid.unwrap(), srvc.id);
                                        }
                                    }
                                }
                                nix::sys::wait::WaitStatus::StillAlive => {
                                    println!("No more state changes to poll");
                                    more_pids = false;
                                }
                                _ => {
                                    println!("Child signaled with code: {:?}", exit_status);
                                }
                            },
                            Err(e) => {
                                if let nix::Error::Sys(nix::errno::ECHILD) = e {
                                    more_pids = false;
                                } else {
                                    println!("Error while waiting: {}", e.description().to_owned());
                                    more_pids = false;
                                }
                            }
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}
