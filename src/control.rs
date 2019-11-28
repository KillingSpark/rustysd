use crate::units::*;
use serde_json::from_str;
use serde_json::Value;

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
                            .push(Value::String(format!("Service: {}", srvc[0].1.conf.name())));
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
                                .push(Value::String(format!("Service: {}", srvc[0].1.conf.name())));
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
                            .push(Value::String(format!("Service: {}", srvc_unit.conf.name())));
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
