use crate::units::*;

use crate::services::{Service, ServiceRuntimeInfo, ServiceStatus};
use crate::sockets::*;

use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::PathBuf;

type ParsedSection = HashMap<String, Vec<(u32, String)>>;
type ParsedFile = HashMap<String, ParsedSection>;

pub fn load_all_units(paths: &[PathBuf]) -> Result<(ServiceTable, SocketTable), String> {
    let mut base_id = 0;
    let mut service_table = HashMap::new();
    let mut socket_unit_table = HashMap::new();
    for path in paths {
        parse_all_services(&mut service_table, path, &mut base_id)?;
        parse_all_sockets(&mut socket_unit_table, path, &mut base_id)?;
    }

    fill_dependencies(&mut service_table);
    let service_table = apply_sockets_to_services(service_table, &socket_unit_table).unwrap();

    open_all_sockets(&mut socket_unit_table).unwrap();
    Ok((service_table, socket_unit_table))
}

fn parse_section(lines: &[&str]) -> ParsedSection {
    let mut entries: ParsedSection = HashMap::new();

    let mut entry_number = 0;
    for line in lines {
        //ignore comments
        if line.starts_with('#') {
            continue;
        }

        //check if this is a key value pair
        let pos = if let Some(pos) = line.find(|c| c == '=') {
            pos
        } else {
            continue;
        };
        let (name, value) = line.split_at(pos);

        let value = value.trim_start_matches('=');
        let value = value.trim();
        let name = name.trim().to_uppercase();
        let values: Vec<String> = value.split(',').map(|x| x.into()).collect();

        let vec = entries.entry(name).or_insert_with(Vec::new);
        for value in values {
            vec.push((entry_number, value));
            entry_number += 1;
        }
    }

    entries
}

fn parse_file(content: &str) -> ParsedFile {
    let mut sections = HashMap::new();
    let lines: Vec<&str> = content.split('\n').collect();

    let mut lines_left = &lines[..];

    // remove lines before the first section
    while !lines_left[0].starts_with('[') {
        lines_left = &lines_left[1..];
    }
    let mut current_section_name: String = lines_left[0].into();
    let mut current_section_lines = Vec::new();

    lines_left = &lines_left[1..];

    while !lines_left.is_empty() {
        let line = lines_left[0];

        if line.starts_with('[') {
            sections.insert(
                current_section_name.clone(),
                parse_section(&current_section_lines),
            );
            current_section_name = line.into();
            current_section_lines.clear();
        } else {
            current_section_lines.push(line);
        }
        lines_left = &lines_left[1..];
    }

    // insert last section
    sections.insert(
        current_section_name.clone(),
        parse_section(&current_section_lines),
    );

    sections
}

fn parse_socket(path: &PathBuf, chosen_id: InternalId) -> Result<Unit, String> {
    let raw = read_to_string(&path)
        .map_err(|e| format!("Error opening file: {:?} error: {}", path, e))?;
    let parsed_file = parse_file(&raw);

    let mut socket_configs = None;
    let mut install_config = None;
    let mut unit_config = None;

    for (name, section) in parsed_file {
        match name.as_str() {
            "[Socket]" => {
                socket_configs = match parse_socket_section(section) {
                    Ok(conf) => Some(conf),
                    Err(e) => return Err(format!("Error in file: {:?} :: {}", path, e)),
                };
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

    // TODO handle install configs for sockets
    let _ = install_config;

    let (sock_name, services, sock_configs) = match socket_configs {
        Some(triple) => triple,
        None => return Err(format!("Didnt find socket config in file: {:?}", path)),
    };

    let conf = match unit_config {
        Some(conf) => conf,
        None => return Err(format!("Didn't find a unit config for file: {:?}", path)),
    };

    Ok(Unit {
        conf,
        id: chosen_id,
        install: Install::default(),
        specialized: UnitSpecialized::Socket(Socket {
            name: sock_name,
            sockets: sock_configs,
            services,
        }),
    })
}

fn parse_service(path: &PathBuf, chosen_id: InternalId) -> Result<Unit, String> {
    let raw = read_to_string(&path)
        .map_err(|e| format!("Error opening file: {:?} error: {}", path, e))?;
    let parsed_file = parse_file(&raw);

    let mut service_config = None;
    let mut install_config = None;
    let mut unit_config = None;

    for (name, section) in parsed_file {
        match name.as_str() {
            "[Service]" => {
                service_config = Some(parse_service_section(section));
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
            socket_names: Vec::new(),

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

fn parse_unix_addr(addr: &str) -> Result<String, ()> {
    if addr.starts_with('/') || addr.starts_with("./") {
        Ok(addr.to_owned())
    } else {
        Err(())
    }
}

fn parse_ipv4_addr(addr: &str) -> Result<std::net::SocketAddrV4, std::net::AddrParseError> {
    let sock: Result<std::net::SocketAddrV4, std::net::AddrParseError> = addr.parse();
    sock
}

fn parse_ipv6_addr(addr: &str) -> Result<std::net::SocketAddrV6, std::net::AddrParseError> {
    let sock: Result<std::net::SocketAddrV6, std::net::AddrParseError> = addr.parse();
    sock
}

fn parse_socket_section(
    section: ParsedSection,
) -> Result<(String, Vec<String>, Vec<SocketConfig>), String> {
    let mut fdname: Option<String> = None;
    let mut socket_kinds: Vec<(u32, SocketKind)> = Vec::new();
    let mut services: Vec<String> = Vec::new();

    // TODO check that there is indeed exactly one value per name
    for (name, mut values) in section {
        match name.as_str() {
            "FILEDESCRIPTORNAME" => {
                fdname = Some(values.remove(0).1);
            }
            "LISTENSTREAM" => {
                for _ in 0..values.len() {
                    let (entry_num, value) = values.remove(0);
                    socket_kinds.push((entry_num, SocketKind::Stream(value)));
                }
            }
            "LISTENDATAGRAM" => {
                for _ in 0..values.len() {
                    let (entry_num, value) = values.remove(0);
                    socket_kinds.push((entry_num, SocketKind::Datagram(value)));
                }
            }
            "LISTENSEQUENTIALPACKET" => {
                for _ in 0..values.len() {
                    let (entry_num, value) = values.remove(0);
                    socket_kinds.push((entry_num, SocketKind::Sequential(value)));
                }
            }
            "SERVICE" => {
                for _ in 0..values.len() {
                    let (_, value) = values.remove(0);
                    services.push(value);
                }
            }
            _ => panic!("Unknown parameter name: {}", name),
        }
    }

    // we need to preserve the original ordering
    socket_kinds.sort_by(|l, r| u32::cmp(&l.0, &r.0));
    let socket_kinds: Vec<SocketKind> = socket_kinds.drain(..).map(|(_, kind)| kind).collect();

    let mut socket_configs = Vec::new();

    for kind in socket_kinds {
        let specialized: SpecializedSocketConfig = match &kind {
            SocketKind::Sequential(addr) => {
                if parse_unix_addr(addr).is_ok() {
                    SpecializedSocketConfig::UnixSocket(UnixSocketConfig { kind: kind.clone() })
                } else {
                    return Err(format!(
                        "No specialized config for socket found for socket addr: {}",
                        addr
                    ));
                }
            }
            SocketKind::Stream(addr) => {
                if parse_unix_addr(addr).is_ok() {
                    SpecializedSocketConfig::UnixSocket(UnixSocketConfig { kind: kind.clone() })
                } else if let Ok(addr) = parse_ipv4_addr(addr) {
                    SpecializedSocketConfig::TcpSocket(TcpSocketConfig {
                        addr: std::net::SocketAddr::V4(addr),
                    })
                } else if let Ok(addr) = parse_ipv6_addr(addr) {
                    SpecializedSocketConfig::TcpSocket(TcpSocketConfig {
                        addr: std::net::SocketAddr::V6(addr),
                    })
                } else {
                    return Err(format!(
                        "No specialized config for socket found for socket addr: {}",
                        addr
                    ));
                }
            }
            SocketKind::Datagram(addr) => {
                if parse_unix_addr(addr).is_ok() {
                    SpecializedSocketConfig::UnixSocket(UnixSocketConfig { kind: kind.clone() })
                } else if let Ok(addr) = parse_ipv4_addr(addr) {
                    SpecializedSocketConfig::UdpSocket(UdpSocketConfig {
                        addr: std::net::SocketAddr::V4(addr),
                    })
                } else if let Ok(addr) = parse_ipv6_addr(addr) {
                    SpecializedSocketConfig::UdpSocket(UdpSocketConfig {
                        addr: std::net::SocketAddr::V6(addr),
                    })
                } else {
                    return Err(format!(
                        "No specialized config for socket found for socket addr: {}",
                        addr
                    ));
                }
            }
        };

        socket_configs.push(SocketConfig {
            kind,
            specialized,
            fd: None,
        });
    }

    let name = match fdname {
        Some(name) => name,
        None => "unknown".into(),
    };

    Ok((name, services, socket_configs))
}

fn map_tupels_to_second<X, Y: Clone>(v: Vec<(X, Y)>) -> Vec<Y> {
    v.iter().map(|(_, scnd)| scnd.clone()).collect()
}

fn string_to_bool(s: &str) -> bool {
    let s_upper = &s.to_uppercase();
    let c: char = s_upper.chars().nth(0).unwrap();

    let is_num_and_zero = s.len() == 1 && c == '0';
    s == "YES" || s == "TRUE" || is_num_and_zero
}

fn parse_unit_section(mut section: ParsedSection, path: &PathBuf) -> UnitConfig {
    let wants = section.remove("WANTS");
    let requires = section.remove("REQUIRES");
    let after = section.remove("AFTER");
    let before = section.remove("BEFORE");

    UnitConfig {
        filepath: path.clone(),
        wants: map_tupels_to_second(wants.unwrap_or_default()),
        requires: map_tupels_to_second(requires.unwrap_or_default()),
        after: map_tupels_to_second(after.unwrap_or_default()),
        before: map_tupels_to_second(before.unwrap_or_default()),
    }
}

fn parse_install_section(mut section: ParsedSection) -> InstallConfig {
    let wantedby = section.remove("WANTEDBY");
    let requiredby = section.remove("REQUIREDBY");

    InstallConfig {
        wanted_by: map_tupels_to_second(wantedby.unwrap_or_default()),
        required_by: map_tupels_to_second(requiredby.unwrap_or_default()),
    }
}

fn parse_service_section(mut section: ParsedSection) -> ServiceConfig {
    let exec = section.remove("EXEC");
    let stop = section.remove("STOP");
    let keep_alive = section.remove("KEEP_ALIVE");
    let sockets = section.remove("SOCKETS");
    let notify_access = section.remove("NOTIFYACCESS");
    let srcv_type = section.remove("TYPE");
    let accept = section.remove("ACCEPT");

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
                panic!("Type had to many entries: {:?}", vec);
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

    let keep_alive = match keep_alive {
        Some(vec) => {
            if vec.len() == 1 {
                string_to_bool(&vec[0].1)
            } else {
                panic!("Keepalive had to many entries: {:?}", vec);
            }
        }
        None => false,
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

    ServiceConfig {
        srcv_type,
        notifyaccess,
        keep_alive,
        accept,
        exec,
        stop,
        sockets: map_tupels_to_second(sockets.unwrap_or_default()),
    }
}

fn get_file_list(path: &PathBuf) -> Result<Vec<std::fs::DirEntry>, String> {
    if !path.exists() {
        return Err(format!("Path to services does not exist: {:?}", path));
    }
    if !path.is_dir() {
        return Err(format!("Path to services does not exist: {:?}", path));
    }
    let mut files: Vec<_> = match std::fs::read_dir(path) {
        Ok(iter) => {
            let files_vec = iter.fold(Ok(Vec::new()), |acc, file| {
                if let Ok(mut files) = acc {
                    match file {
                        Ok(f) => {
                            files.push(f);
                            Ok(files)
                        }
                        Err(e) => Err(format!("Couldnt read dir entry: {}", e)),
                    }
                } else {
                    acc
                }
            });
            match files_vec {
                Ok(files) => files,
                Err(e) => return Err(e),
            }
        }
        Err(e) => return Err(format!("Error while reading dir: {}", e)),
    };
    files.sort_by(|l, r| l.path().cmp(&r.path()));

    Ok(files)
}

pub fn parse_all_services(
    services: &mut std::collections::HashMap<InternalId, Unit>,
    path: &PathBuf,
    last_id: &mut InternalId,
) -> Result<(), String> {
    let files = get_file_list(path)?;
    for entry in files {
        if entry.path().is_dir() {
            parse_all_services(services, path, last_id)?;
        } else if entry.path().to_str().unwrap().ends_with(".service") {
            trace!("{:?}", entry.path());
            *last_id += 1;
            services.insert(*last_id, parse_service(&entry.path(), *last_id)?);
        }
    }
    Ok(())
}

pub fn parse_all_sockets(
    sockets: &mut std::collections::HashMap<InternalId, Unit>,
    path: &PathBuf,
    last_id: &mut InternalId,
) -> Result<(), String> {
    let files = get_file_list(path)?;
    for entry in files {
        if entry.path().is_dir() {
            parse_all_sockets(sockets, path, last_id)?;
        } else if entry.path().to_str().unwrap().ends_with(".socket") {
            trace!("{:?}", entry.path());
            *last_id += 1;
            sockets.insert(*last_id, parse_socket(&entry.path(), *last_id)?);
        }
    }

    Ok(())
}
