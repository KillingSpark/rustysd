use crate::sockets::Socket;
use crate::units::*;
use serde_json::from_str;
use serde_json::{Value};

pub enum Command {
    ListUnits(Option<UnitSpecialized>),
    Status(Option<String>),
}

pub fn parse_command(cmd: &str) -> Result<Command, String> {
    let cmd = from_str(cmd).map_err(|e| format!("Error while decoding json: {}", e))?;
    let command: Command = match cmd {
        Value::Object(map) => {
            let cmd_str = map.get("cmd");
            match cmd_str {
                Some(Value::String(cmd_str)) => match cmd_str.as_str() {
                    "status" => Command::Status(None),
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

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
pub fn execute_command(
    cmd: Command,
    service_table: Arc<Mutex<HashMap<InternalId, Unit>>>,
    socket_table: Arc<Mutex<HashMap<String, Socket>>>,
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
                            .push(Value::String(format!(
                                "Service: {}",
                                srvc[0].1.conf.name()
                            )));
                    } else {
                        if name.ends_with(".socket") {
                            let name = name.trim_end_matches(".socket");
                            let socket_table_locked = socket_table.lock().unwrap();
                            let sock = socket_table_locked.get(name);
                            match sock {
                                Some(_sock) => {
                                    result_vec
                                        .as_array_mut()
                                        .unwrap()
                                        .push(Value::String(format!("Socket: {}", name)));
                                }
                                None => return Err(format!("No socket found with name: {}", name)),
                            }
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
                                    .push(Value::String(format!(
                                        "Service: {}",
                                        srvc[0].1.conf.name()
                                    )));
                            } else {
                                let sock_name = name.trim_end_matches(".socket");
                                let socket_table_locked = socket_table.lock().unwrap();
                                let sock = socket_table_locked.get(sock_name);
                                match sock {
                                    Some(_sock) => {
                                        result_vec.as_array_mut().unwrap().push(Value::String(
                                            format!("Socket: {}", sock_name),
                                        ));
                                    }
                                    None => {
                                        return Err(format!(
                                            "No service or socket found with name: {}",
                                            name
                                        ))
                                    }
                                }
                            }
                        }
                    }
                }
                None => {
                    //list all
                    let srvc_table_locked = &*service_table.lock().unwrap();
                    for (_id, srvc_unit) in srvc_table_locked {
                        result_vec
                            .as_array_mut()
                            .unwrap()
                            .push(Value::String(format!(
                                "Service: {}",
                                srvc_unit.conf.name()
                            )));
                    }
                    let socket_table_locked = &*socket_table.lock().unwrap();
                    for (name, _sock) in socket_table_locked {
                        result_vec
                            .as_array_mut()
                            .unwrap()
                            .push(Value::String(format!("Socket: {}", name)));
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
