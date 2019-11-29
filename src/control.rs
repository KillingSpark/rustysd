use crate::units::*;
use serde_json::Value;
use crate::services;

pub enum Command {
    ListUnits(Option<UnitSpecialized>),
    Status(Option<String>),
}

pub fn parse_command(cmd: Value) -> Result<Command, String> {
    let command: Command = match cmd {
        Value::Object(map) => {
            let cmd_str = map.get("cmd");
            match cmd_str {
                Some(Value::String(cmd_str)) => match cmd_str.as_str() {
                    "status" => match map.get("name") {
                        Some(Value::String(name)) => Command::Status(Some(name.clone())),
                        _ => Command::Status(None),
                    },
                    "list-units" => Command::ListUnits(None),
                    _ => return Err(format!("Unknown command: {}", cmd_str)),
                },
                _ => return Err("No cmd field found".to_owned()),
            }
        }
        _ => return Err("Should have been an object".to_owned()),
    };

    Ok(command)
}

pub fn format_service(srvc_unit: &Unit) -> Value {
    let mut map = serde_json::Map::new();
    map.insert(
        "Name".into(),
        Value::String(format!("{}", srvc_unit.conf.name())),
    );
    if let UnitSpecialized::Service(srvc) = &srvc_unit.specialized {
        map.insert(
            "Sockets".into(),
            Value::Array(
                srvc.socket_names
                    .iter()
                    .map(|x| Value::String(x.clone()))
                    .collect(),
            ),
        );
        let status_str = match srvc.status {
            services::ServiceStatus::NeverRan => "NeverRan".into(),
            services::ServiceStatus::Running => "Running".into(),
            services::ServiceStatus::Starting => "Starting".into(),
            services::ServiceStatus::Stopped => "Stopped".into(),
        };
        map.insert("Status".into(), Value::String(status_str));
        if let Some(instant) = srvc.runtime_info.up_since {
        map.insert("UpSince".into(), Value::String(format!("{:?}", instant.elapsed())));
        }
        map.insert("Restarted".into(), Value::String(format!("{:?}", srvc.runtime_info.restarted)));
    }
    Value::Object(map)
}

pub fn execute_command(
    cmd: Command,
    service_table: ArcMutServiceTable,
    socket_table: ArcMutSocketTable,
) -> Result<String, String> {
    let mut result_vec = Value::Array(Vec::new());
    match cmd {
        Command::Status(unit_name) => {
            match unit_name {
                Some(name) => {
                    //list specific
                    if name.ends_with(".service") {
                        let name = name.trim_end_matches(".service");
                        let srvc_table_locked = service_table.lock().unwrap();
                        let srvc: Vec<_> = srvc_table_locked
                            .iter()
                            .filter(|(_id, unit)| unit.conf.name() == name)
                            .collect();
                        if srvc.len() != 1 {
                            return Err(format!("No service found with name: {}", name));
                        }

                        result_vec
                            .as_array_mut()
                            .unwrap()
                            .push(format_service(&srvc[0].1));
                    } else if name.ends_with(".socket") {
                        let name = name.trim_end_matches(".socket");
                        let socket_table_locked = socket_table.lock().unwrap();
                        let sock: Vec<_> = socket_table_locked
                            .iter()
                            .filter(|(_id, unit)| unit.conf.name() == name)
                            .collect();
                        if sock.len() != 1 {
                            return Err(format!("No socket found with name: {}", name));
                        }

                        result_vec
                            .as_array_mut()
                            .unwrap()
                            .push(Value::String(format!("Socket: {}", sock[0].1.conf.name())));
                    } else {
                        let srvc_name = name.trim_end_matches(".service");
                        let srvc_table_locked = service_table.lock().unwrap();
                        let srvc: Vec<_> = srvc_table_locked
                            .iter()
                            .filter(|(_id, unit)| unit.conf.name() == srvc_name)
                            .collect();
                        if srvc.len() == 1 {
                            result_vec
                                .as_array_mut()
                                .unwrap()
                                .push(format_service(&srvc[0].1));
                        } else {
                            let sock_name = name.trim_end_matches(".socket");
                            let socket_table_locked = socket_table.lock().unwrap();
                            let sock: Vec<_> = socket_table_locked
                                .iter()
                                .filter(|(_id, unit)| unit.conf.name() == sock_name)
                                .collect();
                            if sock.len() != 1 {
                                return Err(format!("No socket found with name: {}", sock_name));
                            }

                            result_vec
                                .as_array_mut()
                                .unwrap()
                                .push(Value::String(format!("Socket: {}", sock[0].1.conf.name())));
                        }
                    }
                }
                None => {
                    //list all
                    let srvc_table_locked = &*service_table.lock().unwrap();
                    for srvc_unit in srvc_table_locked.values() {
                        result_vec
                            .as_array_mut()
                            .unwrap()
                            .push(format_service(&srvc_unit));
                    }
                    let socket_table_locked = &*socket_table.lock().unwrap();
                    for sock_unit in socket_table_locked.values() {
                        result_vec
                            .as_array_mut()
                            .unwrap()
                            .push(Value::String(format!("Socket: {}", sock_unit.conf.name())));
                    }
                }
            }
        }
        Command::ListUnits(_kind) => {
            // list units of kind or all
        }
    }

    Ok(serde_json::to_string_pretty(&result_vec).unwrap())
}

use std::io::Read;
use std::io::Write;
pub fn listen_on_commands<T: 'static + Read + Write + Send>(
    mut source: Box<T>,
    service_table: ArcMutServiceTable,
    socket_table: ArcMutSocketTable,
) {
    std::thread::spawn(move || loop {
        match serde_json::from_reader(&mut *source) {
            Ok(v) => {
                let v: Value = v;
                let cmd = parse_command(v).unwrap();
                let response =
                    execute_command(cmd, service_table.clone(), socket_table.clone()).unwrap();
                source.write_all(response.as_bytes()).unwrap();
            }
            Err(e) => {
                error!("Error while reading from command source {}", e);
                return;
            }
        }
    });
}

pub fn accept_control_connections(
    service_table: ArcMutServiceTable,
    socket_table: ArcMutSocketTable,
) {
    std::thread::spawn(move || {
        use std::os::unix::net::UnixListener;
        let path: std::path::PathBuf = "./notifications/control.socket".into();
        if path.exists() {
            std::fs::remove_file(&path).unwrap();
        }
        let cmd_source = UnixListener::bind(&path).unwrap();
        loop {
            let stream = Box::new(cmd_source.accept().unwrap().0);
            listen_on_commands(stream, service_table.clone(), socket_table.clone())
        }
    });
}
