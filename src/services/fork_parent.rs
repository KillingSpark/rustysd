use crate::services::Service;
use crate::units::*;
use std::os::unix::net::UnixDatagram;

pub fn wait_for_service(
    srvc: &mut Service,
    name: &str,
    stream: &UnixDatagram,
) -> Result<(), String> {
    trace!(
        "[FORK_PARENT] Service: {} forked with pid: {}",
        name,
        srvc.pid.unwrap()
    );

    if let Some(conf) = &srvc.service_config {
        match conf.srcv_type {
            ServiceType::Notify => {
                trace!(
                    "[FORK_PARENT] Waiting for a notification for service {}",
                    name
                );

                let start_time = std::time::Instant::now();
                //let duration_timeout = Some(std::time::Duration::from_nanos(1_000_000_000_000));
                let duration_timeout = None;
                let mut buf = [0u8; 512];
                loop {
                    if let Some(duration_timeout) = duration_timeout {
                        let duration_elapsed = start_time.elapsed();
                        if duration_elapsed > duration_timeout {
                            //TODO handle timeout correctly
                            trace!("[FORK_PARENT] Service {} notification timed out", name);
                            break;
                        } else {
                            let duration_till_timeout = duration_timeout - duration_elapsed;
                            stream
                                .set_read_timeout(Some(duration_till_timeout))
                                .unwrap();
                        }
                    }
                    let bytes = match stream.recv(&mut buf[..]) {
                        Ok(bytes) => bytes,
                        Err(e) => match e.kind() {
                            std::io::ErrorKind::WouldBlock => 0,
                            _ => panic!("{}", e),
                        },
                    };
                    srvc.notifications_buffer
                        .push_str(&String::from_utf8(buf[..bytes].to_vec()).unwrap());
                    crate::notification_handler::handle_notifications_from_buffer(srvc, &name);
                    if srvc.signaled_ready {
                        srvc.signaled_ready = false;
                        trace!("[FORK_PARENT] Service {} sent READY=1 notification", name);
                        break;
                    } else {
                        trace!("[FORK_PARENT] Service {} still not ready", name);
                    }
                }
                stream.set_read_timeout(None).unwrap();
            }
            ServiceType::Simple => {
                trace!("[FORK_PARENT] service {} doesnt notify", name);
            }
            ServiceType::Dbus => {
                if let Some(dbus_name) = &conf.dbus_name {
                    trace!("[FORK_PARENT] Waiting for dbus name: {}", dbus_name);
                    match crate::dbus_wait::wait_for_name_system_bus(
                        &dbus_name,
                        std::time::Duration::from_millis(10_000),
                    ) {
                        Ok(res) => {
                            match res {
                                crate::dbus_wait::WaitResult::Ok => {
                                    trace!("[FORK_PARENT] Found dbus name on bus: {}", dbus_name);
                                }
                                crate::dbus_wait::WaitResult::Timedout => {
                                    warn!(
                                        "[FORK_PARENT] Did not find dbus name on bus: {}",
                                        dbus_name
                                    );
                                    // TODO do something about that
                                }
                            }
                        }
                        Err(e) => {
                            error!("Error while waiting for dbus name: {}", e);
                        }
                    }
                } else {
                    error!("[FORK_PARENT] No busname given for service: {:?}", name);
                }
            }
        }
    }
    Ok(())
}
