use crate::services::{Service, ServiceRuntimeInfo, ServiceStatus};
use crate::units::*;
use std::path::PathBuf;

pub fn parse_service(
    parsed_file: ParsedFile,
    path: &PathBuf,
    chosen_id: InternalId,
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
            status: ServiceStatus::NeverRan,

            service_config,
            socket_ids: Vec::new(),

            process_group: None,

            status_msgs: Vec::new(),

            runtime_info: ServiceRuntimeInfo {
                restarted: 0,
                up_since: None,
            },

            notifications: None,
            stdout_dup: None,
            stderr_dup: None,
            notifications_buffer: String::new(),
        }),
    })
}

fn parse_service_section(mut section: ParsedSection) -> Result<ServiceConfig, ParsingError> {
    let exec = section.remove("EXEC");
    let stop = section.remove("STOP");
    let restart = section.remove("RESTART");
    let sockets = section.remove("SOCKETS");
    let notify_access = section.remove("NOTIFYACCESS");
    let srcv_type = section.remove("TYPE");
    let accept = section.remove("ACCEPT");
    let dbus_name = section.remove("BUSNAME");

    if !section.is_empty() {
        panic!(
            "Service section has unrecognized/unimplemented options: {:?}",
            section
        );
    }

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
        Some(mut vec) => {
            if vec.len() == 1 {
                vec.remove(0).1
            } else {
                panic!("Stop had to many entries: {:?}", vec);
            }
        }
        None => "".to_string(),
    };

    let restart = match restart {
        Some(vec) => {
            if vec.len() == 1 {
                match vec[0].1.to_uppercase().as_str() {
                    "ALWAYS" => ServiceRestart::Always,
                    "NO" => ServiceRestart::No,
                    unknown_setting => panic!("Restart had to unknown setting: {}", unknown_setting),
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
        srcv_type,
        notifyaccess,
        restart,
        accept,
        dbus_name,
        exec,
        stop,
        sockets: map_tupels_to_second(sockets.unwrap_or_default()),
    })
}
