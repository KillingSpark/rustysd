use crate::sockets::Socket;
use std::collections::HashMap;
use std::error::Error;
use std::process::{Command, Stdio};
use threadpool::ThreadPool;

use crate::units::*;

#[derive(Clone)]
pub enum ServiceStatus {
    NeverRan,
    Starting,
    Running,
    Stopped,
}

#[derive(Clone)]
pub struct Service {
    pub pid: Option<u32>,
    pub service_config: Option<ServiceConfig>,

    pub status: ServiceStatus,
    pub sockets: Vec<String>,
}

pub fn kill_services(ids_to_kill: Vec<InternalId>, service_table: &mut HashMap<InternalId, Unit>) {
    //TODO killall services that require this service
    for id in ids_to_kill {
        let unit = service_table.get_mut(&id).unwrap();
        if let UnitSpecialized::Service(srvc) = &unit.specialized {
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
                        unit.conf.name(),
                        srvc.pid
                    );
                }
                Err(e) => panic!(e.description().to_owned()),
            }
        }
    }
}

pub fn service_exit_handler(
    pid: i32,
    code: i8,
    service_table: &mut HashMap<InternalId, Unit>,
    pid_table: &mut HashMap<u32, InternalId>,
    sockets: &HashMap<String, Socket>,
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

    trace!(
        "Service with id: {} pid: {} exited with code: {}",
        srvc_id,
        pid,
        code
    );

    let unit = service_table.get_mut(&srvc_id).unwrap();
    if let UnitSpecialized::Service(srvc) = &mut unit.specialized {
        pid_table.remove(&(pid as u32));
        srvc.status = ServiceStatus::Stopped;

        if let Some(conf) = &srvc.service_config {
            if conf.keep_alive {
                start_service(srvc, unit.conf.name(), sockets);
                pid_table.insert(srvc.pid.unwrap(), unit.id);
            } else {
                trace!(
                    "Killing all services requiring service with id {}: {:?}",
                    srvc_id,
                    unit.install.required_by
                );
                kill_services(unit.install.required_by.clone(), service_table);
            }
        }
    }
}

use std::sync::Arc;
use std::sync::Mutex;
fn run_services_recursive(
    ids_to_start: Vec<InternalId>,
    services: Arc<Mutex<HashMap<InternalId, Unit>>>,
    pids: Arc<Mutex<HashMap<u32, InternalId>>>,
    sockets: Arc<Mutex<HashMap<String, Socket>>>,
    tpool: Arc<Mutex<ThreadPool>>,
    waitgroup: crossbeam::sync::WaitGroup,
) {
    for id in ids_to_start {
        let waitgroup_copy = waitgroup.clone();
        let tpool_copy = Arc::clone(&tpool);
        let services_copy = Arc::clone(&services);
        let pids_copy = Arc::clone(&pids);
        let sockets_copy = Arc::clone(&sockets);

        let job = move || {
            let mut unit = {
                let mut services_locked = services_copy.lock().unwrap();
                services_locked.get_mut(&id).unwrap().clone()
            };
            let name = unit.conf.name();
            if let UnitSpecialized::Service(srvc) = &mut unit.specialized {
                match srvc.status {
                    ServiceStatus::NeverRan => {
                        {
                            let sockets_lock = sockets_copy.lock().unwrap();
                            start_service(srvc, name, &sockets_lock);
                        }
                        let new_pid = srvc.pid.unwrap();
                        {
                            let mut services_locked = services_copy.lock().unwrap();
                            services_locked.insert(id, unit.clone()).unwrap().clone()
                        };
                        {
                            let mut pids = pids_copy.lock().unwrap();
                            pids.insert(new_pid, unit.id);
                        }
                    }
                    _ => unreachable!(),
                }

                run_services_recursive(
                    unit.install.before.clone(),
                    Arc::clone(&services_copy),
                    Arc::clone(&pids_copy),
                    Arc::clone(&sockets_copy),
                    Arc::clone(&tpool_copy),
                    waitgroup_copy,
                );
            }
        };

        {
            let tpool_locked = tpool.lock().unwrap();
            tpool_locked.execute(job);
        }
    }
    drop(waitgroup);
}

pub fn run_services(
    services: HashMap<InternalId, Unit>,
    pids: HashMap<u32, InternalId>,
    sockets: HashMap<String, Socket>,
) -> (HashMap<InternalId, Unit>, HashMap<u32, InternalId>) {
    let mut root_services = Vec::new();

    for (id, unit) in &services {
        if unit.install.after.len() == 0 {
            root_services.push(*id);
        }
    }

    let pool_arc = Arc::new(Mutex::new(ThreadPool::new(6)));
    let services_arc = Arc::new(Mutex::new(services));
    let pids_arc = Arc::new(Mutex::new(pids));
    let sockets_arc = Arc::new(Mutex::new(sockets));
    let waitgroup = crossbeam::sync::WaitGroup::new();
    run_services_recursive(
        root_services,
        Arc::clone(&services_arc),
        Arc::clone(&pids_arc),
        sockets_arc,
        pool_arc,
        waitgroup.clone(),
    );

    waitgroup.wait();

    let services = services_arc.as_ref().lock().unwrap().clone();
    let pids = pids_arc.as_ref().lock().unwrap().clone();

    return (services, pids);
}

fn start_service_with_filedescriptors(
    srvc: &mut Service,
    name: String,
    sockets: &HashMap<String, Socket>,
) {
    // 1. fork
    // 2. in fork use dup2 to map all relevant file desrciptors to 3..x
    // 3. in fork mark all other file descriptors with FD_CLOEXEC
    // 4. set relevant env varibales $LISTEN_FDS $LISTEN_PID
    // 4. execve the cmd with the args

    match nix::unistd::fork() {
        Ok(nix::unistd::ForkResult::Parent { child, .. }) => {
            srvc.pid = Some(child as u32);
            srvc.status = ServiceStatus::Running;

            trace!("Service: {} forked with pid: {}", name, srvc.pid.unwrap());
        }
        Ok(nix::unistd::ForkResult::Child) => {
            let pid = nix::unistd::getpid();

            //here we are in the child process. We need to close every file descriptor we dont need anymore after the exec

            // TODO maybe all fd's should be marked with FD_CLOEXEC when openend
            // and here we only unflag those that we want to keep?
            trace!("CLOSING FDS");

            
            for (name, sock) in sockets {
                trace!("CLOSE FDS FOR SOCKET: {}", name);
                if !srvc.sockets.contains(&name) {
                    for conf in &sock.sockets {
                        match &conf.fd {
                            Some(fd) => {
                                let fd: i32 = (**fd).as_raw_fd();
                                nix::unistd::close(fd).unwrap();
                                trace!("DO CLOSE FD");
                            }
                            None => {
                                //this should not happen but if it does its not too bad
                            }
                        }
                    }
                } else {
                    trace!("DONT CLOSE FD");
                }
            }

            // The following two lines do deadlock after fork and before exec... I would have loved to just use these
            // This has probably something to do with the global env_lock() that is being used in the std
            // std::env::set_var("LISTEN_FDS", format!("{}", srvc.file_descriptors.len()));
            // std::env::set_var("LISTEN_PID", format!("{}", pid));

            // so lets use some unsafe instead, and use the same libc::setenv that the std uses but we dont care about the lock
            // This is the only thread in this process that is still running so we dont need any lock

            // TODO Maybe it would be better to have a simple wrapper that we can exec with a few sensible args
            // 1. list filedescriptors to keep open (maybe not event that. FD handling can be done here probably?)
            // 2. at least the number of fds
            // 3. the actual executable that should be run + their args
            //
            // This wrapper then does:
            // 1. Maybe close and dup2 fds
            // 2. Set appropriate env variables
            // 3. exec the actual executable we are trying to start here

            // This is all just that complicated because systemd promises to pass the correct PID in the env-var LISTEN_PID...

            let mut num_fds = 0;
            let mut name_lists = Vec::new();
            for sock_name in &srvc.sockets {
                let sock = sockets.get(sock_name).unwrap();
                num_fds += sock.sockets.len();
                name_lists.push(sock.build_name_list());
            }

            let pid_str = &format!("{}", pid);
            let fds_str = &format!("{}", num_fds);

            unsafe fn setenv(key: &str, value: &str) {
                let k = std::ffi::CString::new(key.as_bytes()).unwrap();
                let v = std::ffi::CString::new(value.as_bytes()).unwrap();

                libc::setenv(k.as_ptr(), v.as_ptr(), 1);
            }
            let full_name_list = name_lists.join(":");
            unsafe {
                setenv("LISTEN_FDS", fds_str);
            }
            unsafe {
                setenv("LISTEN_PID", pid_str);
            }
            unsafe {
                setenv("LISTEN_FDNAMES", &full_name_list);
            }

            trace!(
                "pid: {}, ENV: LISTEN_PID: {}  LISTEN_FD: {}, LISTEN_FDNAMES: {}",
                pid,
                pid_str,
                fds_str,
                full_name_list
            );

            // no more logging after this point!
            // The filedescriptor used by the logger might have been duped to another
            // one and logging into that one would be.... bad
            // Hopefully the close() means that no old logs will get written to that filedescriptor

            // start at 3. 0,1,2 are stdin,stdout,stderr
            let file_desc_offset = 3;
            let mut fd_idx = 0;
            for sock_name in &srvc.sockets {
                let socket = sockets.get(sock_name).unwrap();
                for sock_conf in &socket.sockets {
                    let new_fd = file_desc_offset + fd_idx;
                    let old_fd = match &sock_conf.fd {
                        Some(fd) => fd.as_raw_fd(),
                        None => panic!("No fd found for socket conf"),
                    };
                    if new_fd as i32 != old_fd {
                        //ignore output. newfd might already be closed
                        let _ = nix::unistd::close(new_fd as i32);
                        nix::unistd::dup2(old_fd, new_fd as i32).unwrap();
                    }
                    fd_idx += 1;
                }
            }

            let split: Vec<&str> = match &srvc.service_config {
                Some(conf) => conf.exec.split(" ").collect(),
                None => unreachable!(),
            };

            let cmd = std::ffi::CString::new(split[0]).unwrap();
            let mut args = Vec::new();
            for arg in &split[1..] {
                args.push(std::ffi::CString::new(*arg).unwrap());
            }

            match nix::unistd::execv(&cmd, &args) {
                Ok(_) => {
                    eprintln!("execv returned Ok()... This should never happen");
                }
                Err(e) => {
                    eprintln!("execv errored: {:?}", e);
                }
            }
        }
        Err(_) => println!("Fork for service: {} failed", name),
    }
}

pub fn start_service(srvc: &mut Service, name: String, sockets: &HashMap<String, Socket>) {
    srvc.status = ServiceStatus::Starting;

    let split: Vec<&str> = match &srvc.service_config {
        Some(conf) => conf.exec.split(" ").collect(),
        None => return,
    };

    if srvc.sockets.len() > 0 {
        start_service_with_filedescriptors(srvc, name, sockets);
    } else {
        let mut cmd = Command::new(split[0]);
        for part in &split[1..] {
            cmd.arg(part);
        }

        cmd.stdout(Stdio::null());

        match cmd.spawn() {
            Ok(child) => {
                srvc.pid = Some(child.id());
                srvc.status = ServiceStatus::Running;

                trace!("Service: {} started with pid: {}", name, srvc.pid.unwrap());
            }
            Err(e) => panic!(e.description().to_owned()),
        }
    }
}

pub fn print_all_services(services: &HashMap<InternalId, Unit>) {
    for (id, unit) in services {
        trace!("{}:", id);
        trace!("  {}", unit.conf.name());
        trace!("  Before {:?}", unit.install.before);
        trace!("  After {:?}", unit.install.after);
    }
}
