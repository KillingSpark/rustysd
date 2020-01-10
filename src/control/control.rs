use crate::units::*;
use serde_json::Value;

#[derive(Debug)]
pub enum Command {
    ListUnits(Option<UnitIdKind>),
    Status(Option<String>),
    Restart(String),
    LoadNew(String),
    Stop(String),
    Shutdown,
}

enum ParseError {
    MethodNotFound(String),
    ParamsInvalid(String),
}

fn parse_command(call: &super::jsonrpc2::Call) -> Result<Command, ParseError> {
    let command = match call.method.as_str() {
        "status" => {
            let name = match &call.params {
                Some(params) => match params {
                    Value::String(s) => Some(s.clone()),
                    _ => {
                        return Err(ParseError::ParamsInvalid(format!(
                            "Params must be either none or a single string"
                        )))
                    }
                },
                None => None,
            };
            Command::Status(name)
        }
        "restart" => {
            let name = match &call.params {
                Some(params) => match params {
                    Value::String(s) => s.clone(),
                    _ => {
                        return Err(ParseError::ParamsInvalid(format!(
                            "Params must be a single string"
                        )))
                    }
                },
                None => {
                    return Err(ParseError::ParamsInvalid(format!(
                        "Params must be a single string"
                    )))
                }
            };
            Command::Restart(name)
        }
        "stop" => {
            let name = match &call.params {
                Some(params) => match params {
                    Value::String(s) => s.clone(),
                    _ => {
                        return Err(ParseError::ParamsInvalid(format!(
                            "Params must be a single string"
                        )))
                    }
                },
                None => {
                    return Err(ParseError::ParamsInvalid(format!(
                        "Params must be a single string"
                    )))
                }
            };
            Command::Stop(name)
        }
        // TODO parse filters for the listing
        "list-units" => {
            let kind = match &call.params {
                Some(params) => match params {
                    Value::String(s) => {
                        let kind = match s.as_str() {
                            "target" => UnitIdKind::Target,
                            "socket" => UnitIdKind::Socket,
                            "service" => UnitIdKind::Service,
                            _ => {
                                return Err(ParseError::ParamsInvalid(format!(
                                    "Kind not recognized: {}",
                                    s
                                )))
                            }
                        };
                        Some(kind)
                    }
                    _ => {
                        return Err(ParseError::ParamsInvalid(format!(
                            "Params must be a single string"
                        )))
                    }
                },
                None => None,
            };
            Command::ListUnits(kind)
        }
        "shutdown" => Command::Shutdown,
        "enable" => {
            let name = match &call.params {
                Some(params) => match params {
                    Value::String(s) => s.clone(),
                    _ => {
                        return Err(ParseError::ParamsInvalid(format!(
                            "Params must be a single string"
                        )))
                    }
                },
                None => {
                    return Err(ParseError::ParamsInvalid(format!(
                        "Params must be a single string"
                    )))
                }
            };
            Command::LoadNew(name)
        }
        _ => {
            return Err(ParseError::MethodNotFound(format!(
                "Unknown method: {}",
                call.method
            )))
        }
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
                srvc.socket_names
                    .iter()
                    .map(|x| Value::String(x.clone()))
                    .collect(),
            ),
        );
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

use std::sync::{Arc, Mutex};
fn find_unit_with_name(unit_name: &str, unit_table_locked: &UnitTable) -> Option<Arc<Mutex<Unit>>> {
    trace!("Find unit for name: {}", unit_name);
    let mut srvc: Vec<_> = unit_table_locked
        .values()
        .filter(|unit| {
            let name = unit.lock().unwrap().conf.name();
            trace!("Name: {}", name);
            unit_name.starts_with(&name) && unit.lock().unwrap().is_service()
        })
        .cloned()
        .collect();
    if srvc.len() != 1 {
        None
    } else {
        Some(srvc.remove(0))
    }
}

// TODO make this some kind of regex pattern matching
fn find_units_with_pattern(
    name_pattern: &str,
    unit_table_locked: &UnitTable,
) -> Vec<Arc<Mutex<Unit>>> {
    trace!("Find units matching pattern: {}", name_pattern);
    let units: Vec<_> = unit_table_locked
        .values()
        .filter(|unit| {
            let name = unit.lock().unwrap().conf.name();
            trace!("Name: {}", name);
            name_pattern.starts_with(&name) && unit.lock().unwrap().is_service()
        })
        .cloned()
        .collect();
    units
}

pub fn execute_command(
    cmd: Command,
    run_info: ArcRuntimeInfo,
    notification_socket_path: std::path::PathBuf,
) -> Result<serde_json::Value, String> {
    let mut result_vec = Value::Array(Vec::new());
    match cmd {
        Command::Shutdown => {
            crate::signal_handler::shutdown_sequence(run_info);
        }
        Command::Restart(unit_name) => {
            let id = if let Some(unit) =
                find_unit_with_name(&unit_name, &*run_info.unit_table.read().unwrap())
            {
                unit.lock().unwrap().id
            } else {
                return Err(format!("No unit found with name: {}", unit_name));
            };

            crate::units::reactivate_unit(
                id,
                run_info,
                notification_socket_path,
                std::sync::Arc::new(Vec::new()),
            )?;
        }
        Command::Stop(unit_name) => {
            let id = if let Some(unit) =
                find_unit_with_name(&unit_name, &*run_info.unit_table.read().unwrap())
            {
                unit.lock().unwrap().id
            } else {
                return Err(format!("No unit found with name: {}", unit_name));
            };

            crate::units::deactivate_unit_recursive(id, true, run_info);
        }
        Command::Status(unit_name) => {
            match unit_name {
                Some(name) => {
                    //list specific
                    let unit_table_locked = &*run_info.unit_table.read().unwrap();
                    let units = find_units_with_pattern(&name, unit_table_locked);
                    for unit in units {
                        if name.ends_with(".service") {
                            result_vec
                                .as_array_mut()
                                .unwrap()
                                .push(format_service(&unit.lock().unwrap()));
                        } else if name.ends_with(".socket") {
                            result_vec
                                .as_array_mut()
                                .unwrap()
                                .push(format_socket(&unit.lock().unwrap()));
                        } else if name.ends_with(".target") {
                            result_vec
                                .as_array_mut()
                                .unwrap()
                                .push(format_target(&unit.lock().unwrap()));
                        } else {
                            return Err("Name suffix not recognized".into());
                        }
                    }
                }
                None => {
                    //list all
                    let unit_table_locked = run_info.unit_table.read().unwrap();
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
        Command::ListUnits(kind) => {
            // TODO list units of kind or all
            let unit_table_locked = run_info.unit_table.read().unwrap();
            for (id, unit) in unit_table_locked.iter() {
                let include = if let Some(kind) = kind {
                    id.0 == kind
                } else {
                    true
                };
                if include {
                    let unit_locked = unit.lock().unwrap();
                    result_vec
                        .as_array_mut()
                        .unwrap()
                        .push(Value::String(unit_locked.conf.name()));
                }
            }
        }
        Command::LoadNew(name) => {
            let this_id = {
                let last_id = &mut *run_info.last_id.lock().unwrap();
                *last_id = *last_id + 1;
                *last_id
            };
            let unit = load_new_unit(&run_info.config.unit_dirs, &name, this_id)?;
            insert_new_unit(unit, run_info)?;
        }
    }

    Ok(result_vec)
}

use std::io::Read;
use std::io::Write;
pub fn listen_on_commands<T: 'static + Read + Write + Send>(
    mut source: Box<T>,
    run_info: ArcRuntimeInfo,
    notification_socket_path: std::path::PathBuf,
) {
    std::thread::spawn(move || loop {
        match super::jsonrpc2::get_next_call(source.as_mut()) {
            Err(e) => {
                if let serde_json::error::Category::Eof = e.classify() {
                    // ignore, just stop reading
                } else {
                    let err = super::jsonrpc2::make_error(
                        super::jsonrpc2::PARSE_ERROR,
                        format!("{}", e),
                        None,
                    );
                    let msg = super::jsonrpc2::make_error_response(None, err);
                    let response_string = serde_json::to_string_pretty(&msg).unwrap();
                    source.write_all(response_string.as_bytes()).unwrap();
                }
                return;
            }
            Ok(call) => {
                match call {
                    Err(e) => {
                        let err = super::jsonrpc2::make_error(
                            super::jsonrpc2::INVALID_REQUEST_ERROR,
                            e,
                            None,
                        );
                        let msg = super::jsonrpc2::make_error_response(None, err);
                        let response_string = serde_json::to_string_pretty(&msg).unwrap();
                        source.write_all(response_string.as_bytes()).unwrap();
                    }
                    Ok(call) => {
                        match parse_command(&call) {
                            Err(e) => {
                                // TODO invalid arguments error
                                let (code, err_msg) = match e {
                                    ParseError::ParamsInvalid(s) => {
                                        (super::jsonrpc2::INVALID_PARAMS_ERROR, s)
                                    }
                                    ParseError::MethodNotFound(s) => {
                                        (super::jsonrpc2::METHOD_NOT_FOUND_ERROR, s)
                                    }
                                };
                                let err = super::jsonrpc2::make_error(code, err_msg, None);
                                let msg = super::jsonrpc2::make_error_response(call.id, err);
                                let response_string = serde_json::to_string_pretty(&msg).unwrap();
                                source.write_all(response_string.as_bytes()).unwrap();
                            }
                            Ok(cmd) => {
                                trace!("Execute command: {:?}", cmd);
                                let msg = match execute_command(
                                    cmd,
                                    run_info.clone(),
                                    notification_socket_path.clone(),
                                ) {
                                    Err(e) => {
                                        let err = super::jsonrpc2::make_error(
                                            super::jsonrpc2::SERVER_ERROR,
                                            e,
                                            None,
                                        );
                                        super::jsonrpc2::make_error_response(call.id, err)
                                    }
                                    Ok(result) => {
                                        super::jsonrpc2::make_result_response(call.id, result)
                                    }
                                };
                                let response_string = serde_json::to_string_pretty(&msg).unwrap();
                                source.write_all(response_string.as_bytes()).unwrap();
                            }
                        }
                    }
                }
            }
        }
    });
}

pub fn accept_control_connections_unix_socket(
    run_info: ArcRuntimeInfo,
    notification_socket_path: std::path::PathBuf,
    source: std::os::unix::net::UnixListener,
) {
    std::thread::spawn(move || {
        let stream = Box::new(source.accept().unwrap().0);
        listen_on_commands(stream, run_info, notification_socket_path.clone())
    });
}

pub fn accept_control_connections_tcp(
    run_info: ArcRuntimeInfo,
    notification_socket_path: std::path::PathBuf,
    source: std::net::TcpListener,
) {
    std::thread::spawn(move || loop {
        let stream = Box::new(source.accept().unwrap().0);
        listen_on_commands(stream, run_info.clone(), notification_socket_path.clone())
    });
}
