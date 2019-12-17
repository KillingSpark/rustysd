use crate::units::*;
use serde_json::Value;

#[derive(Debug)]
pub enum Command {
    ListUnits(Option<UnitSpecialized>),
    Status(Option<String>),
}

pub fn parse_command(cmd: Value) -> Result<Command, String> {
    let command = match cmd {
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

pub fn format_socket(socket_unit: &Unit) -> Value {
    let mut map = serde_json::Map::new();
    map.insert("Name".into(), Value::String(socket_unit.conf.name()));

    if let UnitSpecialized::Socket(sock) = &socket_unit.specialized {
        map.insert(
            "FileDescriptorname".into(),
            Value::String(sock.name.clone()),
        );
        map.insert(
            "FileDescriptors".into(),
            Value::Array(
                sock.sockets
                    .iter()
                    .map(|sock_conf| Value::String(format!("{:?}", sock_conf.specialized)))
                    .collect(),
            ),
        );
    }

    Value::Object(map)
}

pub fn format_target(socket_unit: &Unit) -> Value {
    let mut map = serde_json::Map::new();
    map.insert("Name".into(), Value::String(socket_unit.conf.name()));
    Value::Object(map)
}

pub fn format_service(srvc_unit: &Unit) -> Value {
    let mut map = serde_json::Map::new();
    map.insert("Name".into(), Value::String(srvc_unit.conf.name()));
    if let UnitSpecialized::Service(srvc) = &srvc_unit.specialized {
        map.insert(
            "Sockets".into(),
            Value::Array(
                srvc.socket_ids
                    .iter()
                    .map(|x| Value::String(x.to_string()))
                    .collect(),
            ),
        );
        map.insert("Status".into(), Value::String(srvc.status.to_string()));
        if let Some(instant) = srvc.runtime_info.up_since {
            map.insert(
                "UpSince".into(),
                Value::String(format!("{:?}", instant.elapsed())),
            );
        }
        map.insert(
            "Restarted".into(),
            Value::String(format!("{:?}", srvc.runtime_info.restarted)),
        );
    }
    Value::Object(map)
}

pub fn execute_command(cmd: Command, unit_table: ArcMutUnitTable) -> Result<String, String> {
    let mut result_vec = Value::Array(Vec::new());
    match cmd {
        Command::Status(unit_name) => {
            match unit_name {
                Some(name) => {
                    //list specific
                    if name.ends_with(".service") {
                        let name = name.trim_end_matches(".service");
                        let unit_table_locked = unit_table.read().unwrap();
                        let mut srvc: Vec<_> = unit_table_locked
                            .iter()
                            .filter(|(_id, unit)| unit.lock().unwrap().conf.name() == name)
                            .map(|(_id, unit)| format_service(&unit.lock().unwrap()))
                            .collect();
                        if srvc.len() != 1 {
                            return Err(format!("No service found with name: {}", name));
                        }

                        result_vec.as_array_mut().unwrap().push(srvc.remove(0));
                    } else if name.ends_with(".socket") {
                        let name = name.trim_end_matches(".socket");
                        let unit_table_locked = unit_table.read().unwrap();
                        let mut sock: Vec<_> = unit_table_locked
                            .iter()
                            .filter(|(_id, unit)| unit.lock().unwrap().conf.name() == name)
                            .map(|(_id, unit)| format_socket(&unit.lock().unwrap()))
                            .collect();
                        if sock.len() != 1 {
                            return Err(format!("No service found with name: {}", name));
                        }

                        result_vec.as_array_mut().unwrap().push(sock.remove(0));
                    } else {
                        // name was already short
                        let unit_table_locked = unit_table.read().unwrap();
                        let mut unit: Vec<_> = unit_table_locked
                            .iter()
                            .filter(|(_id, unit)| unit.lock().unwrap().conf.name() == name)
                            .map(|(_id, unit)| {
                                let unit_locked = &unit.lock().unwrap();
                                match unit_locked.specialized {
                                    UnitSpecialized::Socket(_) => format_socket(&unit_locked),
                                    UnitSpecialized::Service(_) => format_service(&unit_locked),
                                    UnitSpecialized::Target => format_target(&unit_locked),
                                }
                            })
                            .collect();
                        if unit.len() != 1 {
                            return Err(format!("No unit found with name: {}", name));
                        }

                        result_vec.as_array_mut().unwrap().push(unit.remove(0));
                    }
                }
                None => {
                    //list all
                    let unit_table_locked = unit_table.read().unwrap();
                    let strings: Vec<_> = unit_table_locked
                        .iter()
                        .map(|(_id, unit)| {
                            let unit_locked = &unit.lock().unwrap();
                            match unit_locked.specialized {
                                UnitSpecialized::Socket(_) => format_socket(&unit_locked),
                                UnitSpecialized::Service(_) => format_service(&unit_locked),
                                UnitSpecialized::Target => format_target(&unit_locked),
                            }
                        })
                        .collect();
                    for s in strings {
                        result_vec.as_array_mut().unwrap().push(s);
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
    unit_table: ArcMutUnitTable,
) {
    std::thread::spawn(move || loop {
        match serde_json::from_reader(&mut *source) {
            Ok(v) => {
                let v: Value = v;
                let cmd = parse_command(v).unwrap();
                let response = execute_command(cmd, unit_table.clone()).unwrap();
                source.write_all(response.as_bytes()).unwrap();
            }
            Err(e) => {
                error!("Error while reading from command source {}", e);
                return;
            }
        }
    });
}

pub fn accept_control_connections(unit_table: ArcMutUnitTable) {
    std::thread::spawn(move || {
        use std::os::unix::net::UnixListener;
        let path: std::path::PathBuf = "./notifications/control.socket".into();
        if path.exists() {
            std::fs::remove_file(&path).unwrap();
        }
        let cmd_source = UnixListener::bind(&path).unwrap();
        loop {
            let stream = Box::new(cmd_source.accept().unwrap().0);
            listen_on_commands(stream, unit_table.clone())
        }
    });
}
