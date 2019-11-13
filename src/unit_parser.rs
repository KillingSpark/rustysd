use crate::services::{
    InstallConfig, InternalId, Service, ServiceConfig, ServiceStatus, UnitConfig,
};
use std::fs::read_to_string;
use std::path::PathBuf;

fn parse_service(path: &PathBuf, chosen_id: InternalId) -> Service {
    let raw = read_to_string(&path).unwrap();
    let lines: Vec<&str> = raw.split("\n").collect();

    let mut service_config = None;
    let mut install_config = None;
    let mut unit_config = None;

    let mut current_section = Vec::new();
    let mut current_section_name = "";
    for idx in 0..lines.len() {
        let line = lines[idx];
        if line.starts_with("[") {
            match current_section_name {
                "" => { /*noting. first section to be found*/ }
                "[Service]" => {
                    service_config = Some(parse_service_section(&current_section));
                }
                "[Unit]" => {
                    unit_config = Some(parse_unit_section(&current_section));
                }
                "[Install]" => {
                    install_config = Some(parse_install_section(&current_section));
                }

                _ => panic!("Unknown section name: {}", current_section_name),
            }
            current_section_name = line;
            current_section.clear();
        } else {
            current_section.push(line);
        }
    }

    //parse last section
    match current_section_name {
        "" => { /*noting. first section to be found*/ }
        "[Service]" => {
            service_config = Some(parse_service_section(&current_section));
        }
        "[Unit]" => {
            unit_config = Some(parse_unit_section(&current_section));
        }
        "[Install]" => {
            install_config = Some(parse_install_section(&current_section));
        }

        _ => panic!("Unknown section name: {}", current_section_name),
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
    }
}

fn parse_unit_section(lines: &Vec<&str>) -> UnitConfig {
    let mut wants = Vec::new();
    let mut requires = Vec::new();
    let mut after = Vec::new();
    let mut before = Vec::new();

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

        match name.as_str() {
            "AFTER" => {
                after.append(&mut values);
            }
            "BEFORE" => {
                before.append(&mut values);
            }
            "WANTS" => {
                wants.append(&mut values);
            }
            "REQUIRES" => {
                requires.append(&mut values);
            }
            _ => panic!("Unknown parameter name"),
        }
    }

    UnitConfig {
        wants: wants,
        requires: requires,
        after: after,
        before: before,
    }
}

fn parse_install_section(lines: &Vec<&str>) -> InstallConfig {
    let mut wantedby = Vec::new();
    let mut requiredby = Vec::new();

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

        match name.as_str() {
            "WANTEDBY" => {
                wantedby.append(&mut values);
            }
            "REQUIREDBY" => {
                requiredby.append(&mut values);
            }
            _ => panic!("Unknown parameter name"),
        }
    }

    InstallConfig {
        wanted_by: wantedby,
        required_by: requiredby,
    }
}

fn parse_service_section(lines: &Vec<&str>) -> ServiceConfig {
    let mut exec = None;
    let mut stop = None;
    let mut keep_alive = None;

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

        match name.as_str() {
            "EXEC" => {
                exec = Some(value.to_owned());
            }
            "STOP" => {
                stop = Some(value.to_owned());
            }
            "KEEP_ALIVE" => {
                keep_alive = Some(value == "true");
            }
            _ => panic!("Unknown parameter name"),
        }
    }

    ServiceConfig {
        keep_alive: keep_alive.unwrap_or(false),
        exec: exec.unwrap_or("".to_owned()),
        stop: stop.unwrap_or("".to_owned()),
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
                println!("{:?}", entry.path());
                *last_id += 1;
                services.insert(*last_id, parse_service(&entry.path(), *last_id));
            }
        }
    }
}
