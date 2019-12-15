use crate::services::{Service, ServiceStatus};
use crate::units::*;
use std::os::unix::net::UnixDatagram;

pub fn after_fork_parent(
    srvc: &mut Service,
    name: String,
    new_pid: nix::unistd::Pid,
    notify_socket_env_var: &std::path::Path,
    stream: &UnixDatagram,
) {
    srvc.pid = Some(new_pid);

    trace!(
        "[FORK_PARENT] Service: {} forked with pid: {}",
        name,
        srvc.pid.unwrap()
    );

    if let Some(conf) = &srvc.service_config {
        match conf.srcv_type {
            ServiceType::Notify => {
                trace!(
                    "[FORK_PARENT] Waiting for a notification on: {:?}",
                    &notify_socket_env_var
                );

                let mut buf = [0u8; 512];
                loop {
                    let bytes = stream.recv(&mut buf[..]).unwrap();
                    srvc.notifications_buffer
                        .push_str(&String::from_utf8(buf[..bytes].to_vec()).unwrap());
                    crate::notification_handler::handle_notifications_from_buffer(srvc, &name);
                    if let ServiceStatus::Running = srvc.status {
                        trace!("[FORK_PARENT] Service {} sent READY=1 notification", name);
                        break;
                    } else {
                        trace!("[FORK_PARENT] Service {} still not ready", name);
                    }
                }
            }
            ServiceType::Simple => {
                trace!("[FORK_PARENT] service {} doesnt notify", name);
                srvc.status = ServiceStatus::Running;
            }
            ServiceType::Dbus => {
                if let Some(dbus_name) = &conf.dbus_name {
                    trace!("[FORK_PARENT] Waiting for dbus name: {}", dbus_name);
                    match crate::dbus_wait::wait_for_name_system_bus(
                        &dbus_name,
                        std::time::Duration::from_millis(10_000),
                    )
                    .unwrap()
                    {
                        crate::dbus_wait::WaitResult::Ok => {
                            trace!("[FORK_PARENT] Found dbus name on bus: {}", dbus_name);
                        }
                        crate::dbus_wait::WaitResult::Timedout => {
                            warn!("[FORK_PARENT] Did not find dbus name on bus: {}", dbus_name);
                            // TODO do something about that
                        }
                    }
                } else {
                    error!("[FORK_PARENT] No busname given for service: {:?}", name);
                }
            }
        }
    }
}