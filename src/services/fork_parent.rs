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
        let start_time = std::time::Instant::now();
        let duration_timeout = {
            if let Some(timeout) = &conf.starttimeout {
                match timeout {
                    Timeout::Duration(dur) => Some(*dur),
                    Timeout::Infinity => None,
                }
            } else {
                if let Some(timeout) = &conf.generaltimeout {
                    match timeout {
                        Timeout::Duration(dur) => Some(*dur),
                        Timeout::Infinity => None,
                    }
                } else {
                    // TODO add default timeout if neither starttimeout nor generaltimeout was set
                    None
                }
            }
        };
        match conf.srcv_type {
            ServiceType::Notify => {
                trace!(
                    "[FORK_PARENT] Waiting for a notification for service {}",
                    name
                );

                //let duration_timeout = Some(std::time::Duration::from_nanos(1_000_000_000_000));
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
            ServiceType::OneShot => {
                trace!(
                    "[FORK_PARENT] Waiting for oneshot service to exit: {}",
                    name
                );
                let mut counter = 1u64;
                loop {
                    if let Some(time_out) = duration_timeout {
                        if start_time.elapsed() >= time_out {
                            //TODO handle timeout correctly
                            error!("oneshot service {} reached timeout", name);
                            break;
                        }
                    }
                    let wait_flags = nix::sys::wait::WaitPidFlag::WNOHANG;
                    let res = nix::sys::wait::waitpid(srvc.pid.unwrap(), Some(wait_flags));
                    match res {
                        Ok(exit_status) => match exit_status {
                            nix::sys::wait::WaitStatus::Exited(_pid, _code) => {
                                // Happy
                                // TODO check exit codes
                                return Ok(());
                            }
                            nix::sys::wait::WaitStatus::Signaled(_pid, _signal, _dumped_core) => {
                                // Happy
                                // TODO check exit codes
                                return Ok(());
                            }
                            nix::sys::wait::WaitStatus::StillAlive => {
                                // Happy but need to wait longer
                            }
                            _ => {
                                // Happy but need to wait longer, we dont care about other events like stop/continue of children
                            }
                        },
                        Err(e) => {
                            if let nix::Error::Sys(nix::errno::Errno::ECHILD) = e {
                                // This might also happen because it was collected by the signal_handler.
                                // This could be fixed by using the waitid() with WNOWAIT in the signal handler but
                                // that has not been ported to rust
                                // in any case it probably means that the process has exited...
                                return Ok(());
                            } else {
                                return Err(format!(
                                    "Error while waiting on oneshot service {}: {}",
                                    name, e
                                ));
                            }
                        }
                    }
                    // exponential backoff to get low latencies for fast processes
                    // but not hog the cpu for too long
                    // start at 0.05 ms
                    // capped to 10 ms to not introduce too big latencies
                    // TODO review those numbers
                    let sleep_dur = std::time::Duration::from_micros(counter * 50);
                    let sleep_cap = std::time::Duration::from_millis(10);
                    let sleep_dur = sleep_dur.min(sleep_cap);
                    if sleep_dur < sleep_cap {
                        counter = counter * 2;
                    }
                    std::thread::sleep(sleep_dur);
                }
            }
            ServiceType::Dbus => {
                if let Some(dbus_name) = &conf.dbus_name {
                    trace!("[FORK_PARENT] Waiting for dbus name: {}", dbus_name);
                    let timeout = if let Some(dur) = duration_timeout {
                        dur
                    }else{
                        // TODO make timeout for dbus optional
                        std::time::Duration::from_secs(1_000_000)
                    };
                    match crate::dbus_wait::wait_for_name_system_bus(
                        &dbus_name,
                        timeout,
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
