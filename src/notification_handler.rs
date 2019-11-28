use crate::services::{Service, ServiceStatus};
use crate::sockets::Socket;
use crate::units::*;
use std::collections::HashMap;
use std::io::Read;
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};

pub fn handle_notification_message(msg: &String, srvc: &mut Service, name: String) {
    // TODO process notification content
    let split: Vec<_> = msg.split("=").collect();
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
            srvc.status = ServiceStatus::Running;
        }
        _ => {
            warn!("Unknown notification name{}", split[0]);
        }
    }
}

fn handle_stream_mut(
    stream: &mut UnixStream,
    id: InternalId,
    service_table: Arc<Mutex<HashMap<InternalId, Unit>>>,
) {
    let mut buffer = String::new();
    loop {
        let mut buf = [0u8; 512];
        let bytes = stream.read(&mut buf[..]).unwrap();
        buffer.push_str(&String::from_utf8(buf[..bytes].to_vec()).unwrap());

        if bytes == 0 {
            // Handle the current buffer and then exit the handler
            let service_table: &mut HashMap<_, _> = &mut service_table.lock().unwrap();
            let srvc_unit = service_table.get_mut(&id).unwrap();
            let name = srvc_unit.conf.name();
            if let UnitSpecialized::Service(srvc) = &mut srvc_unit.specialized {
                handle_notification_message(&buffer, srvc, name);
            }
            trace!(
                " [Notification-Listener] Service: {} closed a notification connection",
                srvc_unit.conf.name(),
            );
            break;
        }
        while buffer.contains('\n') {
            let (line, rest) = buffer.split_at(buffer.find('\n').unwrap());
            let line = line.to_owned();
            buffer = rest[1..].to_owned();

            {
                let service_table: &mut HashMap<_, _> = &mut service_table.lock().unwrap();
                let srvc_unit = service_table.get_mut(&id).unwrap();
                let name = srvc_unit.conf.name();
                if let UnitSpecialized::Service(srvc) = &mut srvc_unit.specialized {
                    handle_notification_message(&line, srvc, name);
                }
            }
        }
    }
}

pub fn handle_stream(
    mut stream: UnixStream,
    id: InternalId,
    service_table: Arc<Mutex<HashMap<InternalId, Unit>>>,
) {
    std::thread::spawn(move || {
        handle_stream_mut(&mut stream, id, service_table);
    });
}

pub fn handle_notifications(
    _socket_table: ArcMutSocketTable,
    service_table: ArcMutServiceTable,
    _pid_table: Arc<Mutex<HashMap<u32, InternalId>>>,
) {
    std::thread::spawn(move || {
        // setup the list to listen to
        let mut select_vec = Vec::new();
        {
            let service_table_locked: &HashMap<_, _> = &service_table.lock().unwrap();
            for (_name, srvc_unit) in service_table_locked {
                if let UnitSpecialized::Service(srvc) = &srvc_unit.specialized {
                    if let Some(sock) = &srvc.notify_access_socket {
                        select_vec.push((srvc_unit.conf.name(), srvc_unit.id, sock.clone()));
                    }
                }
            }
        }

        loop {
            // take refs from the Arc's
            let select_vec: Vec<_> = select_vec
                .iter()
                .map(|(n, id, x)| ((n.clone(), id), x.as_ref()))
                .collect();
            let streams = crate::unix_listener_select::select(&select_vec, None).unwrap();
            for ((name, id), (stream, _addr)) in streams {
                trace!(
                    " [Notification-Listener] Service: {} has connected on the notification socket",
                    name
                );

                // TODO check notification-access setting for pid an such
                {
                    handle_stream(stream, *id, service_table.clone());
                }
            }
        }
    });
}
