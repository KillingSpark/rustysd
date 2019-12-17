use crate::units::*;
use std::error::Error;
use std::process::{Command, Stdio};

fn run_stop_cmd(unit_locked: &Unit, pid_table: &mut PidTable) {
    if let UnitSpecialized::Service(srvc) = &unit_locked.specialized {
        let split: Vec<&str> = match &srvc.service_config {
            Some(conf) => {
                if conf.stop.is_empty() {
                    return;
                }
                conf.stop.split(' ').collect()
            }
            None => return,
        };

        let mut cmd = Command::new(split[0]);
        for part in &split[1..] {
            cmd.arg(part);
        }
        cmd.stdout(Stdio::null());

        match cmd.spawn() {
            Ok(child) => {
                pid_table.insert(
                    nix::unistd::Pid::from_raw(child.id() as i32),
                    PidEntry::Stop(unit_locked.id),
                );
                trace!(
                    "Stopped Service: {} with pid: {:?}",
                    unit_locked.conf.name(),
                    srvc.pid
                );
            }
            Err(e) => panic!(e.description().to_owned()),
        }
    }
}

pub fn kill_service(id_to_kill: InternalId, unit_table: &UnitTable, pid_table: &mut PidTable) {
    let srvc_unit = unit_table.get(&id_to_kill).unwrap();
    let unit_locked = srvc_unit.lock().unwrap();
    run_stop_cmd(&*unit_locked, pid_table);

    if let UnitSpecialized::Service(srvc) = &unit_locked.specialized {
        if let Some(proc_group) = srvc.process_group {
            match nix::sys::signal::kill(proc_group, nix::sys::signal::Signal::SIGKILL) {
                Ok(_) => trace!(
                    "Success killing process group for service {}",
                    unit_locked.conf.name(),
                ),
                Err(e) => error!(
                    "Error killing process group for service {}: {}",
                    unit_locked.conf.name(),
                    e,
                ),
            }
        }
    }
}

pub fn kill_services(
    ids_to_kill: Vec<InternalId>,
    unit_table: &UnitTable,
    pid_table: &mut PidTable,
) {
    //TODO killall services that require this service
    for id in ids_to_kill {
        kill_service(id, unit_table, pid_table);
    }
}
