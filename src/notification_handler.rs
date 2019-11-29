use crate::services::{Service, ServiceStatus};
use crate::units::*;
use std::collections::HashMap;
use std::os::unix::net::UnixDatagram;
use std::sync::{Arc, Mutex};

pub fn handle_notification_message(msg: &str, srvc: &mut Service, name: &str) {
    // TODO process notification content
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
            srvc.status = ServiceStatus::Running;
        }
        _ => {
            warn!("Unknown notification name{}", split[0]);
        }
    }
}

pub fn handle_notifications_from_buffer(mut buffer: String, srvc: &mut Service, name: &str) -> String {
    while buffer.contains('\n') {
        let (line, rest) = buffer.split_at(buffer.find('\n').unwrap());
        let line = line.to_owned();
        buffer = rest[1..].to_owned();

        handle_notification_message(&line, srvc, name);
    }
    buffer
}

fn handle_stream_mut(
    stream: &mut UnixDatagram,
    id: InternalId,
    service_table: Arc<Mutex<HashMap<InternalId, Unit>>>,
    buf: String,
) {
    let mut buffer = buf;
    let mut buf = [0u8; 512];
    loop {
        {
            let service_table: &mut HashMap<_, _> = &mut service_table.lock().unwrap();
            let srvc_unit = service_table.get_mut(&id).unwrap();
            let name = srvc_unit.conf.name();
            if let UnitSpecialized::Service(srvc) = &mut srvc_unit.specialized {
                buffer = handle_notifications_from_buffer(buffer, srvc, &name);
            }
        }
        let bytes = stream.recv(&mut buf[..]).unwrap();
        buffer.push_str(&String::from_utf8(buf[..bytes].to_vec()).unwrap());

        if bytes == 0 {
            // Handle the current buffer and then exit the handler
            let service_table: &mut HashMap<_, _> = &mut service_table.lock().unwrap();
            let srvc_unit = service_table.get_mut(&id).unwrap();
            let name = srvc_unit.conf.name();
            if let UnitSpecialized::Service(srvc) = &mut srvc_unit.specialized {
                handle_notifications_from_buffer(buffer, srvc, &name);
            }
            trace!(
                " [Notification-Listener] Service: {} closed a notification connection",
                srvc_unit.conf.name(),
            );
            break;
        }
    }
}

pub fn handle_stream(
    mut stream: UnixDatagram,
    id: InternalId,
    service_table: Arc<Mutex<HashMap<InternalId, Unit>>>,
    buf: String,
) {
    std::thread::spawn(move || {
        handle_stream_mut(&mut stream, id, service_table, buf);
    });
}
