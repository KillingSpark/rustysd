//! collect the different streams from the services
//! Stdout and stderr get redirected to the normal stdout/err but are prefixed with a unique string to identify their output
//! streams from the notification sockets get parsed and applied to the respective service

use log::trace;
use log::warn;

use crate::platform::reset_event_fd;
use crate::runtime_info::*;
use crate::services::Service;
use crate::services::StdIo;
use crate::units::*;
use std::{collections::HashMap, os::unix::io::AsRawFd};

fn collect_from_srvc<F>(run_info: ArcMutRuntimeInfo, f: F) -> HashMap<i32, UnitId>
where
    F: Fn(&mut HashMap<i32, UnitId>, &Service, UnitId),
{
    let run_info_locked = run_info.read().unwrap();
    let unit_table = &run_info_locked.unit_table;
    unit_table
        .iter()
        .fold(HashMap::new(), |mut map, (id, srvc_unit)| {
            if let Specific::Service(srvc) = &srvc_unit.specific {
                let state = &*srvc.state.read().unwrap();
                f(&mut map, &state.srvc, id.clone());
            }
            map
        })
}

pub fn handle_all_streams(run_info: ArcMutRuntimeInfo) {
    let eventfd = { run_info.read().unwrap().notification_eventfd };
    loop {
        // need to collect all again. There might be a newly started service
        let fd_to_srvc_id = collect_from_srvc(run_info.clone(), |map, srvc, id| {
            if let Some(socket) = &srvc.notifications {
                map.insert(socket.as_raw_fd(), id);
            }
        });

        let mut fdset = nix::sys::select::FdSet::new();
        for fd in fd_to_srvc_id.keys() {
            fdset.insert(*fd);
        }
        fdset.insert(eventfd.read_end());

        let result = nix::sys::select::select(None, Some(&mut fdset), None, None, None);

        let run_info_locked = run_info.read().unwrap();
        let unit_table = &run_info_locked.unit_table;
        match result {
            Ok(_) => {
                if fdset.contains(eventfd.read_end()) {
                    trace!("Interrupted notification select because the eventfd fired");
                    reset_event_fd(eventfd);
                    trace!("Reset eventfd value");
                }
                let mut buf = [0u8; 512];
                for (fd, id) in &fd_to_srvc_id {
                    if fdset.contains(*fd) {
                        if let Some(srvc_unit) = unit_table.get(id) {
                            if let Specific::Service(srvc) = &srvc_unit.specific {
                                let mut_state = &mut *srvc.state.write().unwrap();
                                if let Some(socket) = &mut_state.srvc.notifications {
                                    let old_flags =
                                        nix::fcntl::fcntl(*fd, nix::fcntl::FcntlArg::F_GETFL)
                                            .unwrap();

                                    let old_flags =
                                        nix::fcntl::OFlag::from_bits(old_flags).unwrap();
                                    let mut new_flags = old_flags.clone();
                                    new_flags.insert(nix::fcntl::OFlag::O_NONBLOCK);
                                    nix::fcntl::fcntl(
                                        *fd,
                                        nix::fcntl::FcntlArg::F_SETFL(new_flags),
                                    )
                                    .unwrap();
                                    let bytes = {
                                        match socket.recv(&mut buf[..]) {
                                            Ok(b) => b,
                                            Err(e) => match e.kind() {
                                                std::io::ErrorKind::WouldBlock => 0,
                                                _ => panic!("{}", e),
                                            },
                                        }
                                    };
                                    nix::fcntl::fcntl(
                                        *fd,
                                        nix::fcntl::FcntlArg::F_SETFL(old_flags),
                                    )
                                    .unwrap();
                                    let note_str =
                                        String::from_utf8(buf[..bytes].to_vec()).unwrap();
                                    mut_state.srvc.notifications_buffer.push_str(&note_str);
                                    crate::notification_handler::handle_notifications_from_buffer(
                                        &mut mut_state.srvc,
                                        &srvc_unit.id.name,
                                    );
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Error while selecting: {}", e);
            }
        }
    }
}

pub fn handle_all_std_out(run_info: ArcMutRuntimeInfo) {
    let eventfd = { run_info.read().unwrap().stdout_eventfd };
    loop {
        // need to collect all again. There might be a newly started service
        let fd_to_srvc_id = collect_from_srvc(run_info.clone(), |map, srvc, id| {
            if let Some(StdIo::Piped(r, _w)) = &srvc.stdout {
                map.insert(*r, id);
            }
        });

        let mut fdset = nix::sys::select::FdSet::new();
        for fd in fd_to_srvc_id.keys() {
            fdset.insert(*fd);
        }
        fdset.insert(eventfd.read_end());

        let result = nix::sys::select::select(None, Some(&mut fdset), None, None, None);

        let run_info_locked = run_info.read().unwrap();
        let unit_table = &run_info_locked.unit_table;
        match result {
            Ok(_) => {
                if fdset.contains(eventfd.read_end()) {
                    trace!("Interrupted stdout select because the eventfd fired");
                    reset_event_fd(eventfd);
                    trace!("Reset eventfd value");
                }
                let mut buf = [0u8; 512];
                for (fd, id) in &fd_to_srvc_id {
                    if fdset.contains(*fd) {
                        if let Some(srvc_unit) = unit_table.get(id) {
                            let name = srvc_unit.id.name.clone();
                            if let Specific::Service(srvc) = &srvc_unit.specific {
                                let mut_state = &mut *srvc.state.write().unwrap();
                                let status = srvc_unit.common.status.read().unwrap();

                                let old_flags =
                                    nix::fcntl::fcntl(*fd, nix::fcntl::FcntlArg::F_GETFL).unwrap();
                                let old_flags = nix::fcntl::OFlag::from_bits(old_flags).unwrap();
                                let mut new_flags = old_flags.clone();
                                new_flags.insert(nix::fcntl::OFlag::O_NONBLOCK);
                                nix::fcntl::fcntl(*fd, nix::fcntl::FcntlArg::F_SETFL(new_flags))
                                    .unwrap();

                                ////
                                let bytes = match nix::unistd::read(*fd, &mut buf[..]) {
                                    Ok(b) => b,
                                    Err(nix::Error::EWOULDBLOCK) => 0,
                                    Err(e) => panic!("{}", e),
                                };
                                ////

                                nix::fcntl::fcntl(*fd, nix::fcntl::FcntlArg::F_SETFL(old_flags))
                                    .unwrap();

                                mut_state.srvc.stdout_buffer.extend(&buf[..bytes]);
                                mut_state.srvc.log_stdout_lines(&name, &status).unwrap();
                            }
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Error while selecting: {}", e);
            }
        }
    }
}

pub fn handle_all_std_err(run_info: ArcMutRuntimeInfo) {
    let eventfd = { run_info.read().unwrap().stderr_eventfd };
    loop {
        // need to collect all again. There might be a newly started service
        let fd_to_srvc_id = collect_from_srvc(run_info.clone(), |map, srvc, id| {
            if let Some(StdIo::Piped(r, _w)) = &srvc.stderr {
                map.insert(*r, id);
            }
        });

        let mut fdset = nix::sys::select::FdSet::new();
        for fd in fd_to_srvc_id.keys() {
            fdset.insert(*fd);
        }
        fdset.insert(eventfd.read_end());

        let result = nix::sys::select::select(None, Some(&mut fdset), None, None, None);
        let run_info_locked = run_info.read().unwrap();
        let unit_table = &run_info_locked.unit_table;

        match result {
            Ok(_) => {
                if fdset.contains(eventfd.read_end()) {
                    trace!("Interrupted stderr select because the eventfd fired");
                    reset_event_fd(eventfd);
                    trace!("Reset eventfd value");
                }
                let mut buf = [0u8; 512];
                for (fd, id) in &fd_to_srvc_id {
                    if fdset.contains(*fd) {
                        if let Some(srvc_unit) = unit_table.get(id) {
                            let name = srvc_unit.id.name.clone();
                            if let Specific::Service(srvc) = &srvc_unit.specific {
                                let mut_state = &mut *srvc.state.write().unwrap();
                                let status = srvc_unit.common.status.read().unwrap();

                                let old_flags =
                                    nix::fcntl::fcntl(*fd, nix::fcntl::FcntlArg::F_GETFL).unwrap();
                                let old_flags = nix::fcntl::OFlag::from_bits(old_flags).unwrap();
                                let mut new_flags = old_flags.clone();
                                new_flags.insert(nix::fcntl::OFlag::O_NONBLOCK);
                                nix::fcntl::fcntl(*fd, nix::fcntl::FcntlArg::F_SETFL(new_flags))
                                    .unwrap();

                                ////
                                let bytes = match nix::unistd::read(*fd, &mut buf[..]) {
                                    Ok(b) => b,
                                    Err(nix::Error::EWOULDBLOCK) => 0,
                                    Err(e) => panic!("{}", e),
                                };
                                ////
                                nix::fcntl::fcntl(*fd, nix::fcntl::FcntlArg::F_SETFL(old_flags))
                                    .unwrap();

                                mut_state.srvc.stderr_buffer.extend(&buf[..bytes]);
                                mut_state.srvc.log_stderr_lines(&name, &status).unwrap();
                            }
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Error while selecting: {}", e);
            }
        }
    }
}

pub fn handle_notification_message(msg: &str, srvc: &mut Service, name: &str) {
    let split: Vec<_> = msg.split('=').collect();
    match split[0] {
        "STATUS" => {
            srvc.status_msgs.push(split[1].to_owned());
            trace!(
                "New status message pushed from service {}: {}",
                name,
                srvc.status_msgs.last().unwrap()
            );
        }
        "READY" => {
            srvc.signaled_ready = true;
        }
        _ => {
            warn!("Unknown notification name{}", split[0]);
        }
    }
}

pub fn handle_notifications_from_buffer(srvc: &mut Service, name: &str) {
    while srvc.notifications_buffer.contains('\n') {
        let (line, rest) = srvc
            .notifications_buffer
            .split_at(srvc.notifications_buffer.find('\n').unwrap());
        let line = line.to_owned();
        srvc.notifications_buffer = rest[1..].to_owned();

        handle_notification_message(&line, srvc, name);
    }
}
