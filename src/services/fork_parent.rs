use log::error;
use log::trace;
use log::warn;

use crate::runtime_info::*;
use crate::services::RunCmdError;
use crate::services::Service;
use crate::units::ServiceConfig;
use crate::units::*;

pub fn wait_for_service(
    srvc: &mut Service,
    conf: &ServiceConfig,
    name: &str,
    run_info: &RuntimeInfo,
) -> Result<(), RunCmdError> {
    let pid_table = &run_info.pid_table;
    trace!(
        "[FORK_PARENT] Service: {} forked with pid: {}",
        name,
        srvc.pid.unwrap()
    );

    let start_time = std::time::Instant::now();
    let duration_timeout = srvc.get_start_timeout(conf);
    match conf.srcv_type {
        ServiceType::Notify => {
            trace!(
                "[FORK_PARENT] Waiting for a notification for service {}",
                name
            );

            //let duration_timeout = Some(std::time::Duration::from_nanos(1_000_000_000_000));
            let mut buf = [0u8; 512];
            loop {
                let stream = if let Some(stream) = &srvc.notifications {
                    stream
                } else {
                    return Err(RunCmdError::Generic(
                        "No notification socket but is required".into(),
                    ));
                };

                {
                    let mut pid_table_locked = pid_table.lock().unwrap();
                    if let Some(PidEntry::ServiceExited(_)) =
                        pid_table_locked.get(&srvc.pid.unwrap())
                    {
                        trace!(
                            "The service {} has exited before sending a READY=1 notification",
                            name
                        );
                        let pid_entry = pid_table_locked.remove(&srvc.pid.unwrap());
                        if let Some(PidEntry::ServiceExited(code)) = pid_entry {
                            return Err(RunCmdError::ExitBeforeNotify(name.to_owned(), code));
                        }
                    }
                }

                if let Some(duration_timeout) = duration_timeout {
                    let duration_elapsed = start_time.elapsed();
                    if duration_elapsed > duration_timeout {
                        trace!("[FORK_PARENT] Service {} notification timed out", name);
                        return Err(RunCmdError::Timeout(
                            conf.exec.to_string(),
                            format!("{:?}", duration_timeout),
                        ));
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
                        std::io::ErrorKind::Interrupted => 0,
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
            if let Some(stream) = &srvc.notifications {
                stream.set_read_timeout(None).unwrap();
            }
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
            let pid = srvc.pid.unwrap();
            loop {
                if let Some(time_out) = duration_timeout {
                    if start_time.elapsed() >= time_out {
                        error!("oneshot service {} reached timeout", name);
                        return Err(RunCmdError::Timeout(
                            conf.exec.to_string(),
                            format!("{:?}", duration_timeout),
                        ));
                    }
                }
                {
                    let mut pid_table_locked = pid_table.lock().unwrap();
                    match pid_table_locked.get(&pid) {
                        Some(entry) => {
                            match entry {
                                PidEntry::Service(_, _) => {
                                    // Still running. Wait more
                                }
                                PidEntry::ServiceExited(_) => {
                                    trace!("End wait for {}", name);
                                    let entry_owned = pid_table_locked.remove(&pid).unwrap();
                                    if let PidEntry::ServiceExited(code) = entry_owned {
                                        if !code.success() {
                                            if !conf
                                                .exec
                                                .prefixes
                                                .contains(&CommandlinePrefix::Minus)
                                            {
                                                return Err(RunCmdError::BadExitCode(
                                                    conf.exec.to_string(),
                                                    code,
                                                ));
                                            }
                                        }
                                    }
                                    break;
                                }
                                PidEntry::Helper(_, _) => {
                                    // Should never happen
                                    unreachable!(
                                            "Was waiting on oneshot process but pid got saved as PidEntry::Helper"
                                        );
                                }
                                PidEntry::HelperExited(_) => {
                                    // Should never happen
                                    unreachable!(
                                            "Was waiting on oneshot process but pid got saved as PidEntry::HelperExited"
                                        );
                                }
                            }
                        }
                        None => {
                            // Should not happen. Either there is an Helper entry oder a Exited entry
                            unreachable!("No entry for child found")
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
                match crate::dbus_wait::wait_for_name_system_bus(&dbus_name, duration_timeout) {
                    Ok(res) => match res {
                        crate::dbus_wait::WaitResult::Ok => {
                            trace!("[FORK_PARENT] Found dbus name on bus: {}", dbus_name);
                        }
                        crate::dbus_wait::WaitResult::Timedout => {
                            warn!("[FORK_PARENT] Did not find dbus name on bus: {}", dbus_name);
                            return Err(RunCmdError::Timeout(
                                conf.exec.to_string(),
                                format!("{:?}", duration_timeout),
                            ));
                        }
                    },
                    Err(e) => {
                        return Err(RunCmdError::WaitError(
                            conf.exec.to_string(),
                            format!("Error while waiting for dbus name: {}", e),
                        ));
                    }
                }
            } else {
                return Err(RunCmdError::Generic(format!(
                    "[FORK_PARENT] No busname given for service: {:?}",
                    name
                )));
            }
        }
    }
    Ok(())
}
