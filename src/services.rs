use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use threadpool::ThreadPool;

pub type InternalId = u64;

#[derive(Clone)]
pub struct UnitConfig {
    pub wants: Vec<String>,
    pub requires: Vec<String>,
    pub before: Vec<String>,
    pub after: Vec<String>,
}
#[derive(Clone)]
pub struct InstallConfig {
    pub wanted_by: Vec<String>,
    pub required_by: Vec<String>,
}

#[derive(Clone)]
pub struct ServiceConfig {
    pub keep_alive: bool,
    pub exec: String,
    pub stop: String,
}

#[derive(Clone)]
pub enum ServiceStatus {
    NeverRan,
    Starting,
    Running,
    Stopped,
}

#[derive(Clone)]
pub struct Service {
    pub id: InternalId,
    pub pid: Option<u32>,
    pub filepath: PathBuf,
    pub status: ServiceStatus,

    pub wants: Vec<InternalId>,
    pub requires: Vec<InternalId>,

    pub wanted_by: Vec<InternalId>,
    pub required_by: Vec<InternalId>,

    pub before: Vec<InternalId>,
    pub after: Vec<InternalId>,

    pub service_config: Option<ServiceConfig>,
    pub unit_config: Option<UnitConfig>,
    pub install_config: Option<InstallConfig>,
}

impl Service {
    pub fn name(&self) -> String {
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

    pub fn dedup_dependencies(&mut self) {
        self.wants.dedup();
        self.requires.dedup();
        self.wanted_by.dedup();
        self.required_by.dedup();
        self.before.dedup();
        self.after.dedup();
    }
}

pub fn kill_services(
    ids_to_kill: Vec<InternalId>,
    service_table: &mut HashMap<InternalId, Service>,
) {
    //TODO killall services that require this service
    for id in ids_to_kill {
        let srvc = service_table.get(&id).unwrap();

        let split: Vec<&str> = match &srvc.service_config {
            Some(conf) => {
                if conf.stop.len() == 0 {
                    continue;
                }
                conf.stop.split(" ").collect()
            }
            None => continue,
        };

        let mut cmd = Command::new(split[0]);
        for part in &split[1..] {
            cmd.arg(part);
        }
        cmd.stdout(Stdio::null());

        match cmd.spawn() {
            Ok(_) => {
                trace!(
                    "Stopped Service: {} with pid: {:?}",
                    srvc.name(),
                    srvc.pid
                );
            }
            Err(e) => panic!(e.description().to_owned()),
        }
    }
}

pub fn service_exit_handler(
    pid: i32,
    code: i8,
    service_table: &mut HashMap<InternalId, Service>,
    pid_table: &mut HashMap<u32, InternalId>,
) {
    let srvc_id = *(match pid_table.get(&(pid as u32)) {
        Some(id) => id,
        None => {
            trace!("Ignore event for pid: {}", pid);
            // Probably a kill command
            //TODO track kill command pid's
            return;
        }
    });
    let srvc = service_table.get_mut(&srvc_id).unwrap();

    trace!(
        "Service with id: {} pid: {} exited with code: {}",
        srvc_id,
        pid,
        code
    );

    pid_table.remove(&(pid as u32));
    srvc.status = ServiceStatus::Stopped;

    if let Some(conf) = &srvc.service_config {
        if conf.keep_alive {
            start_service(srvc);
            pid_table.insert(srvc.pid.unwrap(), srvc.id);
        } else {
            trace!(
                "Killing all services requiring service with id {}: {:?}",
                srvc_id,
                srvc.required_by
            );
            kill_services(srvc.required_by.clone(), service_table);
        }
    }
}

use std::sync::Arc;
use std::sync::Mutex;
fn run_services_recursive(
    ids_to_start: Vec<InternalId>,
    services: Arc<Mutex<HashMap<InternalId, Service>>>,
    pids: Arc<Mutex<HashMap<u32, InternalId>>>,
    tpool: Arc<Mutex<ThreadPool>>,
    waitgroup: crossbeam::sync::WaitGroup,
) {
    for id in ids_to_start {
        let waitgroup_copy = waitgroup.clone();
        let tpool_copy = Arc::clone(&tpool);
        let services_copy = Arc::clone(&services);
        let pids_copy = Arc::clone(&pids);

        let job = move || {
            let mut srvc = {
                let mut services_locked = services_copy.lock().unwrap();
                services_locked.get_mut(&id).unwrap().clone()
            };
            match srvc.status {
                ServiceStatus::NeverRan => {
                    start_service(&mut srvc);
                    {
                        let mut services_locked = services_copy.lock().unwrap();
                        services_locked.insert(id, srvc.clone()).unwrap().clone()
                    };
                    {
                        let mut pids = pids_copy.lock().unwrap();
                        pids.insert(srvc.pid.unwrap(), srvc.id);
                    }
                }
                _ => unreachable!(),
            }
            
            run_services_recursive(
                srvc.before.clone(),
                Arc::clone(&services_copy),
                Arc::clone(&pids_copy),
                Arc::clone(&tpool_copy),
                waitgroup_copy
            );
        };

        {
            let tpool_locked = tpool.lock().unwrap();
            tpool_locked.execute(job);
        }
    }
    drop(waitgroup);
}

pub fn run_services(
    services: HashMap<InternalId, Service>,
    pids: HashMap<u32, InternalId>,
) -> (HashMap<InternalId, Service>, HashMap<u32, InternalId>) {
    let mut root_services = Vec::new();

    for (id, srvc) in &services {
        if srvc.after.len() == 0 {
            root_services.push(*id);
        }
    }

    let pool_arc = Arc::new(Mutex::new(ThreadPool::new(6)));
    let services_arc = Arc::new(Mutex::new(services));
    let pids_arc = Arc::new(Mutex::new(pids));
    let waitgroup = crossbeam::sync::WaitGroup::new();
    run_services_recursive(
        root_services,
        Arc::clone(&services_arc),
        Arc::clone(&pids_arc),
        pool_arc,
        waitgroup.clone(),
    );

    waitgroup.wait();

    
    let services = services_arc.as_ref().lock().unwrap().clone();
    let pids = pids_arc.as_ref().lock().unwrap().clone();

    return (services, pids);
}

pub fn start_service(srvc: &mut Service) {
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

            trace!(
                "Service: {} started with pid: {}",
                srvc.name(),
                srvc.pid.unwrap()
            );
        }
        Err(e) => panic!(e.description().to_owned()),
    }
}

pub fn print_all_services(services: &HashMap<InternalId, Service>) {
    for (id, srvc) in services {
        trace!("{}:", id);
        trace!("  {}", srvc.name());
        trace!("  Before {:?}", srvc.before);
        trace!("  After {:?}", srvc.after);
    }
}

pub fn fill_dependencies(services: &mut HashMap<InternalId, Service>) -> HashMap<String, u64> {
    let mut name_to_id = HashMap::new();

    for (id, srvc) in &*services {
        let name = srvc.name();
        name_to_id.insert(name, *id);
    }

    let mut required_by = Vec::new();
    let mut wanted_by: Vec<(InternalId, InternalId)> = Vec::new();
    let mut before = Vec::new();
    let mut after = Vec::new();

    for (_, srvc) in &mut *services {
        if let Some(conf) = &srvc.unit_config {
            for name in &conf.wants {
                let id = name_to_id.get(name.as_str()).unwrap();
                srvc.wants.push(*id);
                wanted_by.push((*id, srvc.id));
            }
            for name in &conf.requires {
                let id = name_to_id.get(name.as_str()).unwrap();
                srvc.requires.push(*id);
                required_by.push((*id, srvc.id));
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
                wanted_by.push((srvc.id, *id));
            }
            for name in &conf.required_by {
                let id = name_to_id.get(name.as_str()).unwrap();
                required_by.push((srvc.id, *id));
            }
        }
    }

    for (wanted, wanting) in wanted_by {
        let srvc = services.get_mut(&wanting).unwrap();
        srvc.wants.push(wanted);
        let srvc = services.get_mut(&wanted).unwrap();
        srvc.wanted_by.push(wanting);
    }

    for (required, requiring) in required_by {
        let srvc = services.get_mut(&requiring).unwrap();
        srvc.requires.push(required);
        let srvc = services.get_mut(&required).unwrap();
        srvc.required_by.push(requiring);
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
