use crate::services::{
    InstallConfig, InternalId, Service, ServiceConfig, ServiceStatus, UnitConfig,
};

use crate::sockets::{Socket, SocketConfig, SocketKind, SpecializedSocketConfig, UnixSocketConfig};

use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::PathBuf;

pub enum UnitSpecialized {
    Socket(Socket),
    Service(Service),
}

pub struct Unit {
    pub conf: UnitConfig,
    pub id: InternalId,
    pub specialized: UnitSpecialized,
}

type ParsedSection = HashMap<String, Vec<String>>;
type ParsedFile = HashMap<String, ParsedSection>;

fn parse_section(lines: &Vec<&str>) -> ParsedSection {
    let mut entries: HashMap<String, Vec<String>> = HashMap::new();
    for line in lines {
        let pos = if let Some(pos) = line.find(|c| c == '=') {
            pos
        } else {
            continue;
        };
        let (name, value) = line.split_at(pos);

        let value = value.trim_start_matches("=");
        let value = value.trim();
        let name = name.trim().to_uppercase();
        let mut values: Vec<String> = value.split(",").map(|x| x.to_owned()).collect();

        match entries.get_mut(&name) {
            Some(vec) => vec.append(&mut values),
            None => {
                entries.insert(name, values);
            }
        }
    }

    entries
}

fn parse_file(content: &String) -> ParsedFile {
    let mut sections = HashMap::new();
    let lines: Vec<&str> = content.split("\n").collect();

    let mut lines_left = &lines[..];

    let mut current_section_name = "".to_string();
    let mut current_section_lines = Vec::new();
    while lines_left.len() > 0 {
        for idx in 0..lines_left.len() {
            let line = lines_left[idx];
            println!("{}", line);
            if current_section_name == "" {
                current_section_name = line.into();
                current_section_lines.clear();
            } else {
                if line.starts_with("[") || idx == lines_left.len() - 1 {
                    sections.insert(
                        current_section_name.clone(),
                        parse_section(&current_section_lines),
                    );
                    current_section_name = line.into();
                    current_section_lines.clear();
                    lines_left = &lines_left[idx + 1..];
                    break;
                } else {
                    current_section_lines.push(line.into());
                }
            }
        }
    }

    sections
}

fn parse_socket(path: &PathBuf, chosen_id: InternalId) -> Result<Unit, String> {
    let raw = read_to_string(&path).unwrap();
    let parsed_file = parse_file(&raw);

    let mut socket_configs = Vec::new();
    let mut install_config = None;
    let mut unit_config = None;

    for (name, section) in parsed_file {
        match name.as_str() {
            "[Socket]" => {
                socket_configs = match parse_socket_section(section) {
                    Ok(conf) => conf,
                    Err(e) => return Err(format!("Error in file: {:?} :: {}", path, e)),
                };
            }
            "[Unit]" => {
                unit_config = Some(parse_unit_section(section));
            }
            "[Install]" => {
                install_config = Some(parse_install_section(section));
            }

            _ => panic!("Unknown section name: {}", name),
        }
    }

    // TODO handle install configs for sockets
    let _ = install_config;

    Ok(Unit {
        conf: unit_config.unwrap().clone(),
        id: chosen_id,
        specialized: UnitSpecialized::Socket(Socket {
            filepath: path.clone(),
            sockets: socket_configs
                .iter()
                .map(|conf| (conf.clone(), None))
                .collect(),
        }),
    })
}

fn parse_service(path: &PathBuf, chosen_id: InternalId) -> Service {
    let raw = read_to_string(&path).unwrap();
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
                unit_config = Some(parse_unit_section(section));
            }
            "[Install]" => {
                install_config = Some(parse_install_section(section));
            }

            _ => panic!("Unknown section name: {}", name),
        }
    }

    Service {
        id: chosen_id,
        pid: None,
        filepath: path.clone(),
        status: ServiceStatus::NeverRan,

        wants: Vec::new(),
        wanted_by: Vec::new(),
        requires: Vec::new(),
        required_by: Vec::new(),
        before: Vec::new(),
        after: Vec::new(),
        service_config: service_config,
        unit_config: unit_config,
        install_config: install_config,

        file_descriptors: Vec::new(),
    }
}

fn parse_socket_section(section: ParsedSection) -> Result<Vec<SocketConfig>, String> {
    let mut fdname: Option<String> = None;
    let mut socket_kinds = Vec::new();

    // TODO check that there is indeed exactly one value per name
    for (name, mut values) in section {
        match name.as_str() {
            "FILEDESCRIPTORNAME" => {
                fdname = Some(values.remove(0));
            }
            "LISTENSTREAM" => {
                socket_kinds.push(SocketKind::Stream(values.remove(0)));
            }
            "LISTENDATAGRAM" => {
                socket_kinds.push(SocketKind::Datagram(values.remove(0)));
            }
            "LISTENSEQUENTIALPACKET" => {
                socket_kinds.push(SocketKind::Sequential(values.remove(0)));
            }
            _ => panic!("Unknown parameter name: {}", name),
        }
    }

    let mut socket_configs = Vec::new();

    for kind in socket_kinds {
        let specialized: SpecializedSocketConfig = match &kind {
            SocketKind::Sequential(addr) => {
                if addr.starts_with("/") || addr.starts_with("./") {
                    SpecializedSocketConfig::UnixSocket(UnixSocketConfig {
                        path: addr.clone().into(),
                        listener: None,
                    })
                } else {
                    return Err(format!(
                        "No specialized config for socket found for socket addr: {}",
                        addr
                    )
                    .into());
                }
            }
            SocketKind::Stream(addr) => {
                if addr.starts_with("/") || addr.starts_with("./") {
                    SpecializedSocketConfig::UnixSocket(UnixSocketConfig {
                        path: addr.clone().into(),
                        listener: None,
                    })
                } else {
                    return Err(format!(
                        "No specialized config for socket found for socket addr: {}",
                        addr
                    )
                    .into());
                }
            }
            SocketKind::Datagram(addr) => {
                if addr.starts_with("/") || addr.starts_with("./") {
                    SpecializedSocketConfig::UnixSocket(UnixSocketConfig {
                        path: addr.clone().into(),
                        listener: None,
                    })
                } else {
                    return Err(format!(
                        "No specialized config for socket found for socket addr: {}",
                        addr
                    )
                    .into());
                }
            }
        };

        socket_configs.push(SocketConfig {
            name: match &fdname {
                Some(name) => name.clone(),
                None => "unknown".into(),
            },
            kind: kind,
            specialized: specialized,
        });
    }

    return Ok(socket_configs);
}

fn parse_unit_section(mut section: ParsedSection) -> UnitConfig {
    let wants = section.remove("WANTS");
    let requires = section.remove("REQUIRES");
    let after = section.remove("AFTER");
    let before = section.remove("BEFORE");

    UnitConfig {
        wants: wants.unwrap_or(Vec::new()),
        requires: requires.unwrap_or(Vec::new()),
        after: after.unwrap_or(Vec::new()),
        before: before.unwrap_or(Vec::new()),
    }
}

fn parse_install_section(mut section: ParsedSection) -> InstallConfig {
    let wantedby = section.remove("WANTEDBY");
    let requiredby = section.remove("REQUIREDBY");

    InstallConfig {
        wanted_by: wantedby.unwrap_or(Vec::new()),
        required_by: requiredby.unwrap_or(Vec::new()),
    }
}

fn parse_service_section(mut section: ParsedSection) -> ServiceConfig {
    let exec = section.remove("EXEC");
    let stop = section.remove("STOP");
    let keep_alive = section.remove("KEEP_ALIVE");

    let exec = match exec {
        Some(mut vec) => {
            if vec.len() == 1 {
                vec.remove(0)
            } else {
                panic!("Exec had to many entries: {:?}", vec);
            }
        }
        None => "".to_string(),
    };

    let stop = match stop {
        Some(mut vec) => {
            if vec.len() == 1 {
                vec.remove(0)
            } else {
                panic!("Stop had to many entries: {:?}", vec);
            }
        }
        None => "".to_string(),
    };

    let keep_alive = match keep_alive {
        Some(vec) => {
            if vec.len() == 1 {
                vec[0] == "true"
            } else {
                panic!("Keepalive had to many entries: {:?}", vec);
            }
        }
        None => false,
    };

    ServiceConfig {
        keep_alive: keep_alive,
        exec: exec,
        stop: stop,
    }
}

pub fn parse_all_services(
    services: &mut std::collections::HashMap<InternalId, Service>,
    path: &PathBuf,
    last_id: &mut InternalId,
) {
    let mut files: Vec<_> = std::fs::read_dir(path)
        .unwrap()
        .map(|e| e.unwrap())
        .collect();
    files.sort_by(|l, r| l.path().cmp(&r.path()));
    for entry in files {
        if entry.path().is_dir() {
            parse_all_services(services, path, last_id);
        } else {
            if entry.path().to_str().unwrap().ends_with(".service") {
                trace!("{:?}", entry.path());
                *last_id += 1;
                services.insert(*last_id, parse_service(&entry.path(), *last_id));
            }
        }
    }
}

pub fn parse_all_sockets(
    sockets: &mut std::collections::HashMap<InternalId, Unit>,
    path: &PathBuf,
    last_id: &mut InternalId,
) {
    let mut files: Vec<_> = std::fs::read_dir(path)
        .unwrap()
        .map(|e| e.unwrap())
        .collect();
    files.sort_by(|l, r| l.path().cmp(&r.path()));
    for entry in files {
        if entry.path().is_dir() {
            parse_all_sockets(sockets, path, last_id);
        } else {
            if entry.path().to_str().unwrap().ends_with(".socket") {
                trace!("{:?}", entry.path());
                *last_id += 1;
                sockets.insert(*last_id, parse_socket(&entry.path(), *last_id).unwrap());
            }
        }
    }
}
