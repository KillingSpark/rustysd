use crate::{internalId, Service, ServiceConfig, ServiceStatus};
use std::fs::read_to_string;
use std::path::PathBuf;

fn parse_service(path: &PathBuf, chosen_id: internalId) -> Service {
    let raw = read_to_string(&path).unwrap();
    let lines: Vec<&str> = raw.split("\n").collect();

    let mut config = None;

    let mut current_section = Vec::new();
    let mut current_section_name = "";
    for idx in 0..lines.len() {
        let line = lines[idx];
        if line.starts_with("[") {
            match current_section_name {
                "" => { /*noting. first section to be found*/ }
                "[Service]" => {
                    println!("A");
                    config = Some(parse_service_section(&current_section));
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
            println!("A");
            config = Some(parse_service_section(&current_section));
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
        config: config.unwrap(),
    }
}

fn parse_service_section(lines: &Vec<&str>) -> ServiceConfig {
    let mut exec = None;
    let mut stop = None;
    let mut keep_alive = None;

    for line in lines {
        println!("{}", line);
        let pos = line.find(|c| c == '=').unwrap();
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
    services: &mut std::collections::HashMap<internalId, Service>,
    path: &PathBuf,
    last_id: &mut internalId,
) {
    for entry in std::fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();

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
