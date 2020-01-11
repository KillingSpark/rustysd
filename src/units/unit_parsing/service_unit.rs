use crate::services::{Service, ServiceRuntimeInfo};
use crate::units::*;
use std::path::PathBuf;

pub fn parse_service(
    parsed_file: ParsedFile,
    path: &PathBuf,
    chosen_id: UnitId,
) -> Result<Unit, ParsingError> {
    let mut service_config = None;
    let mut install_config = None;
    let mut unit_config = None;

    for (name, section) in parsed_file {
        match name.as_str() {
            "[Service]" => {
                service_config = Some(parse_service_section(section)?);
            }
            "[Unit]" => {
                unit_config = Some(parse_unit_section(section, path));
            }
            "[Install]" => {
                install_config = Some(parse_install_section(section));
            }

            _ => panic!("Unknown section name: {}", name),
        }
    }

    let service_config = if let Some(service_config) = service_config {
        service_config
    } else {
        return Err(ParsingError::from(format!(
            "Service unit {:?} did not contain a service section",
            path
        )));
    };

    Ok(Unit {
        id: chosen_id,
        conf: unit_config.unwrap_or(UnitConfig {
            filepath: path.clone(),

            description: "".into(),

            wants: Vec::new(),
            requires: Vec::new(),
            before: Vec::new(),
            after: Vec::new(),
        }),
        install: Install {
            wants: Vec::new(),
            wanted_by: Vec::new(),
            requires: Vec::new(),
            required_by: Vec::new(),
            before: Vec::new(),
            after: Vec::new(),
            install_config,
        },
        specialized: UnitSpecialized::Service(Service {
            pid: None,
            signaled_ready: false,

            service_config,
            socket_names: Vec::new(),

            process_group: None,

            status_msgs: Vec::new(),

            runtime_info: ServiceRuntimeInfo {
                restarted: 0,
                up_since: None,
            },

            notifications: None,
            notifications_path: None,
            stdout_dup: None,
            stderr_dup: None,
            notifications_buffer: String::new(),
            stdout_buffer: Vec::new(),
            stderr_buffer: Vec::new(),
        }),
    })
}

fn parse_timeout(descr: &str) -> Timeout {
    if descr.to_uppercase() == "INFINITY" {
        Timeout::Infinity
    } else {
        match descr.parse::<u64>() {
            Ok(secs) => Timeout::Duration(std::time::Duration::from_secs(secs)),
            Err(_) => {
                let mut sum = 0;
                let split = descr.split(' ').collect::<Vec<_>>();
                for t in &split {
                    if t.ends_with("min") {
                        let mins = t[0..t.len() - 3].parse::<u64>().unwrap();
                        sum += mins * 60;
                    } else if t.ends_with("hrs") {
                        let hrs = t[0..t.len() - 3].parse::<u64>().unwrap();
                        sum += hrs * 60 * 60;
                    } else if t.ends_with("s") {
                        let secs = t[0..t.len() - 1].parse::<u64>().unwrap();
                        sum += secs;
                    }
                }
                Timeout::Duration(std::time::Duration::from_secs(sum))
            }
        }
    }
}

fn parse_service_section(mut section: ParsedSection) -> Result<ServiceConfig, ParsingError> {
    let exec = section.remove("EXECSTART");
    let stop = section.remove("EXECSTOP");
    let stoppost = section.remove("EXECSTOPPOST");
    let startpre = section.remove("EXECSTARTPRE");
    let startpost = section.remove("EXECSTARTPOST");
    let starttimeout = section.remove("TIMEOUTSTARTSEC");
    let stoptimeout = section.remove("TIMEOUTSTOPSEC");
    let generaltimeout = section.remove("TIMEOUTSEC");

    let restart = section.remove("RESTART");
    let sockets = section.remove("SOCKETS");
    let notify_access = section.remove("NOTIFYACCESS");
    let srcv_type = section.remove("TYPE");
    let accept = section.remove("ACCEPT");
    let dbus_name = section.remove("BUSNAME");

    let exec_config = super::parse_exec_section(&mut section)?;

    if !section.is_empty() {
        panic!(
            "Service section has unrecognized/unimplemented options: {:?}",
            section
        );
    }

    let starttimeout = match starttimeout {
        Some(vec) => {
            if vec.len() == 1 {
                Some(parse_timeout(&vec[0].1))
            } else {
                panic!("TimeoutStartSec had to many entries: {:?}", vec);
            }
        }
        None => None,
    };
    let stoptimeout = match stoptimeout {
        Some(vec) => {
            if vec.len() == 1 {
                Some(parse_timeout(&vec[0].1))
            } else {
                panic!("TimeoutStopSec had to many entries: {:?}", vec);
            }
        }
        None => None,
    };
    let generaltimeout = match generaltimeout {
        Some(vec) => {
            if vec.len() == 1 {
                Some(parse_timeout(&vec[0].1))
            } else {
                panic!("TimeoutSec had to many entries: {:?}", vec);
            }
        }
        None => None,
    };

    let exec = match exec {
        Some(mut vec) => {
            if vec.len() == 1 {
                vec.remove(0).1
            } else {
                panic!("Exec had to many entries: {:?}", vec);
            }
        }
        None => "".to_string(),
    };

    let srcv_type = match srcv_type {
        Some(vec) => {
            if vec.len() == 1 {
                match vec[0].1.as_str() {
                    "simple" => ServiceType::Simple,
                    "notify" => ServiceType::Notify,
                    "oneshot" => ServiceType::OneShot,
                    "dbus" => {
                        if cfg!(feature = "dbus_support") {
                            ServiceType::Dbus
                        } else {
                            return Err(ParsingError::from(format!("Dbus service found but rustysd was built without the feature dbus_support")));
                        }
                    }
                    _ => panic!("Unknown service type: {}", vec[0].1),
                }
            } else {
                panic!("Type had to many entries: {:?}", vec);
            }
        }
        None => ServiceType::Simple,
    };

    let notifyaccess = match notify_access {
        Some(vec) => {
            if vec.len() == 1 {
                match vec[0].1.as_str() {
                    "all" => NotifyKind::All,
                    "main" => NotifyKind::Main,
                    "exec" => NotifyKind::Exec,
                    "none" => NotifyKind::None,
                    _ => panic!("Unknown notify access: {}", vec[0].1),
                }
            } else {
                panic!("Notifyaccess had to many entries: {:?}", vec);
            }
        }
        None => NotifyKind::Main,
    };

    let stop = match stop {
        Some(vec) => map_tupels_to_second(vec),
        None => Vec::new(),
    };
    let stoppost = match stoppost {
        Some(vec) => map_tupels_to_second(vec),
        None => Vec::new(),
    };
    let startpre = match startpre {
        Some(vec) => map_tupels_to_second(vec),
        None => Vec::new(),
    };
    let startpost = match startpost {
        Some(vec) => map_tupels_to_second(vec),
        None => Vec::new(),
    };

    let restart = match restart {
        Some(vec) => {
            if vec.len() == 1 {
                match vec[0].1.to_uppercase().as_str() {
                    "ALWAYS" => ServiceRestart::Always,
                    "NO" => ServiceRestart::No,
                    unknown_setting => {
                        panic!("Restart had to unknown setting: {}", unknown_setting)
                    }
                }
            } else {
                panic!("Restart had to many entries: {:?}", vec);
            }
        }
        None => ServiceRestart::No,
    };
    let accept = match accept {
        Some(vec) => {
            if vec.len() == 1 {
                string_to_bool(&vec[0].1)
            } else {
                panic!("Accept had to many entries: {:?}", vec);
            }
        }
        None => false,
    };
    let dbus_name = match dbus_name {
        Some(vec) => {
            if vec.len() == 1 {
                Some(vec[0].1.to_owned())
            } else {
                panic!("BusName had to many entries: {:?}", vec);
            }
        }
        None => None,
    };

    if let ServiceType::Dbus = srcv_type {
        if dbus_name.is_none() {
            panic!("BusName not specified but service type is dbus");
        }
    }

    Ok(ServiceConfig {
        exec_config,
        srcv_type,
        notifyaccess,
        restart,
        accept,
        dbus_name,
        exec,
        stop,
        stoppost,
        startpre,
        startpost,
        starttimeout,
        stoptimeout,
        generaltimeout,
        sockets: map_tupels_to_second(sockets.unwrap_or_default()),
    })
}
