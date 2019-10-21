use std::error::Error;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::collections::HashMap;

extern crate signal_hook;
use signal_hook::iterator::Signals;
extern crate libc;

extern crate nix;

type internalId = u64;

struct ServiceConfig {
    keep_alive: bool,
    exec: String,
    stop: String,
}

struct Service {
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

enum ServiceStatus {
    NeverRan,
    Running,
    Stopped,
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
    let s1 = Service {
        id: 0,
        pid: None,
        filepath: PathBuf::from("/usr/lib/systemd/system/dbus.service"),
        status: ServiceStatus::NeverRan,

        wants: Vec::new(),
        wanted_by: Vec::new(),
        requires: Vec::new(),
        required_by: Vec::new(),
        before: Vec::new(),
        after: Vec::new(),

        config: ServiceConfig {
            keep_alive: true,
            exec: "/usr/bin/ls /etc".to_owned(),
            stop: "".to_owned(),
        },
    };

    let signals = Signals::new(&[
        signal_hook::SIGCHLD,
    ]).expect("Couldnt setup listening to the signals");

    let mut service_table = HashMap::new();
    let mut pid_table = HashMap::new();
    service_table.insert(s1.id, s1);

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
                                    println!("Child pid: {} exited with code: {}", pid, code);
                                    let srvc_id = pid_table.get(&(pid as u32)).unwrap();
                                    let srvc = service_table.get_mut(&srvc_id).unwrap();

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
