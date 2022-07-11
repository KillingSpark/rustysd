use crate::runtime_info::*;
use crate::units::*;

use log::trace;
use serde_json::Value;

pub fn open_all_sockets(run_info: ArcMutRuntimeInfo, conf: &crate::config::Config) {
    // TODO make configurable
    let control_sock_path = {
        run_info
            .read()
            .unwrap()
            .config
            .notification_sockets_dir
            .join("control.socket")
    };
    if control_sock_path.exists() {
        std::fs::remove_file(&control_sock_path).unwrap();
    }
    use std::os::unix::net::UnixListener;
    std::fs::create_dir_all(&conf.notification_sockets_dir).unwrap();
    let unixsock = UnixListener::bind(&control_sock_path).unwrap();
    accept_control_connections_unix_socket(run_info.clone(), unixsock);
    //let tcpsock = std::net::TcpListener::bind("127.0.0.1:8080").unwrap();
    //accept_control_connections_tcp(
    //    run_info.clone(),
    //    conf.notification_sockets_dir.clone(),
    //    tcpsock,
    //);
}

#[derive(Debug)]
pub enum Command {
    ListUnits(Option<UnitIdKind>),
    Status(Option<String>),
    LoadNew(Vec<String>),
    LoadAllNew,
    LoadAllNewDry,
    Remove(String),
    Restart(String),
    Start(String),
    StartAll(String),
    Stop(String),
    StopAll(String),
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
        "start" => {
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
            Command::Start(name)
        }
        "start-all" => {
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
            Command::StartAll(name)
        }
        "remove" => {
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
            Command::Remove(name)
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
        "stop-all" => {
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
            Command::StopAll(name)
        }

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
        "reload" => Command::LoadAllNew,
        "reload-dry" => Command::LoadAllNewDry,
        "enable" => {
            let names = match &call.params {
                Some(params) => match params {
                    Value::String(s) => vec![s.clone()],
                    Value::Array(names) => {
                        let mut str_names = Vec::new();
                        for name in names {
                            if let Value::String(name) = name {
                                str_names.push(name.clone());
                            } else {
                                return Err(ParseError::ParamsInvalid(format!(
                                    "Params must be at least one string"
                                )));
                            }
                        }
                        str_names
                    }
                    _ => {
                        return Err(ParseError::ParamsInvalid(format!(
                            "Params must be at least one string"
                        )))
                    }
                },
                None => {
                    return Err(ParseError::ParamsInvalid(format!(
                        "Params must be at least one string"
                    )))
                }
            };
            Command::LoadNew(names)
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

pub fn format_socket(socket_unit: &Unit, status: UnitStatus) -> Value {
    let mut map = serde_json::Map::new();
    map.insert("Name".into(), Value::String(socket_unit.id.name.clone()));
    map.insert("Status".into(), Value::String(format!("{:?}", status)));

    if let Specific::Socket(sock) = &socket_unit.specific {
        map.insert(
            "FileDescriptorname".into(),
            Value::String(socket_unit.id.name.clone()),
        );
        map.insert(
            "FileDescriptors".into(),
            Value::Array(
                sock.conf
                    .sockets
                    .iter()
                    .map(|sock_conf| Value::String(format!("{:?}", sock_conf.specialized)))
                    .collect(),
            ),
        );
    }

    Value::Object(map)
}

pub fn format_target(socket_unit: &Unit, status: UnitStatus) -> Value {
    let mut map = serde_json::Map::new();
    map.insert("Name".into(), Value::String(socket_unit.id.name.clone()));
    map.insert("Status".into(), Value::String(format!("{:?}", status)));
    Value::Object(map)
}

pub fn format_service(srvc_unit: &Unit, status: UnitStatus) -> Value {
    let mut map = serde_json::Map::new();
    map.insert("Name".into(), Value::String(srvc_unit.id.name.clone()));
    map.insert("Status".into(), Value::String(format!("{:?}", status)));
    if let Specific::Service(srvc) = &srvc_unit.specific {
        map.insert(
            "Sockets".into(),
            Value::Array(
                srvc.conf
                    .sockets
                    .iter()
                    .map(|x| Value::String(x.name.clone()))
                    .collect(),
            ),
        );
        if let Some(instant) = srvc.state.read().unwrap().common.up_since {
            map.insert(
                "UpSince".into(),
                Value::String(format!("{:?}", instant.elapsed())),
            );
        }
        map.insert(
            "Restarted".into(),
            Value::String(format!(
                "{:?}",
                srvc.state.read().unwrap().common.restart_count
            )),
        );
    }
    Value::Object(map)
}

fn find_units_with_name<'a>(unit_name: &str, unit_table: &'a UnitTable) -> Vec<&'a Unit> {
    trace!("Find unit for name: {}", unit_name);
    unit_table
        .values()
        .filter(|unit| {
            let name = unit.id.name.clone();
            name.starts_with(&unit_name)
        })
        .collect()
}

// TODO make this some kind of regex pattern matching
fn find_units_with_pattern<'a>(
    name_pattern: &str,
    unit_table_locked: &'a UnitTable,
) -> Vec<&'a Unit> {
    trace!("Find units matching pattern: {}", name_pattern);
    let units: Vec<_> = unit_table_locked
        .values()
        .filter(|unit| {
            let name = unit.id.name.clone();
            name.starts_with(&name_pattern)
        })
        .collect();
    units
}

pub fn execute_command(
    cmd: Command,
    run_info: ArcMutRuntimeInfo,
) -> Result<serde_json::Value, String> {
    let mut result_vec = Value::Array(Vec::new());
    match cmd {
        Command::Shutdown => {
            crate::shutdown::shutdown_sequence(run_info);
        }
        Command::Restart(unit_name) => {
            let run_info = &*run_info.read().unwrap();
            let id = {
                let unit_table = &run_info.unit_table;
                let units = find_units_with_name(&unit_name, unit_table);
                if units.len() > 1 {
                    let names: Vec<_> = units.iter().map(|unit| unit.id.name.clone()).collect();
                    return Err(format!(
                        "More than one unit found with name: {}: {:?}",
                        unit_name, names
                    ));
                }
                if units.len() == 0 {
                    return Err(format!("No unit found with name: {}", unit_name));
                }
                let x = units[0].id.clone();
                x
            };

            match crate::units::reactivate_unit(id, run_info).map_err(|e| format!("{}", e)) {
                Err(e) => {
                    return Err(e);
                }
                Ok(_) => {
                    // Happy
                }
            };
        }
        Command::Start(unit_name) => {
            let run_info = &*run_info.read().unwrap();
            let id = {
                let unit_table = &run_info.unit_table;
                let units = find_units_with_name(&unit_name, unit_table);
                if units.len() > 1 {
                    let names: Vec<_> = units.iter().map(|unit| unit.id.name.clone()).collect();
                    return Err(format!(
                        "More than one unit found with name: {}: {:?}",
                        unit_name, names
                    ));
                }
                if units.len() == 0 {
                    return Err(format!("No unit found with name: {}", unit_name));
                }
                let x = units[0].id.clone();
                x
            };

            match crate::units::activate_unit(id, run_info, ActivationSource::Regular)
                .map_err(|e| format!("{}", e))
            {
                Err(e) => {
                    return Err(e);
                }
                Ok(_) => {
                    // Happy
                }
            };
        }
        Command::StartAll(unit_name) => {
            let id = {
                let run_info_locked = &*run_info.read().unwrap();
                let unit_table = &run_info_locked.unit_table;
                let units = find_units_with_name(&unit_name, unit_table);
                if units.len() > 1 {
                    let names: Vec<_> = units.iter().map(|unit| unit.id.name.clone()).collect();
                    return Err(format!(
                        "More than one unit found with name: {}: {:?}",
                        unit_name, names
                    ));
                }
                if units.len() == 0 {
                    return Err(format!("No unit found with name: {}", unit_name));
                }
                let x = units[0].id.clone();
                x
            };

            let errs = crate::units::activate_needed_units(id, run_info);
            if errs.len() > 0 {
                let mut errstr = String::from("Errors while starting the units:");
                for err in errs {
                    errstr.push_str(&format!("\n{:?}", err));
                }
                return Err(errstr);
            }
        }
        Command::Remove(unit_name) => {
            let run_info = &mut *run_info.write().unwrap();
            let id = {
                let units = find_units_with_name(&unit_name, &run_info.unit_table);
                if units.len() > 1 {
                    let names: Vec<_> = units.iter().map(|unit| unit.id.name.clone()).collect();
                    return Err(format!(
                        "More than one unit found with name: {}: {:?}",
                        unit_name, names
                    ));
                }
                if units.len() == 0 {
                    return Err(format!("No unit found with name: {}", unit_name));
                }
                let x = units[0].id.clone();
                x
            };

            crate::units::remove_unit_with_dependencies(id, run_info)
                .map_err(|e| format!("{}", e))?;
        }
        Command::Stop(unit_name) => {
            let run_info = &*run_info.read().unwrap();
            let id = {
                let units = find_units_with_name(&unit_name, &run_info.unit_table);
                if units.len() > 1 {
                    let names: Vec<_> = units.iter().map(|unit| unit.id.name.clone()).collect();
                    return Err(format!(
                        "More than one unit found with name: {}: {:?}",
                        unit_name, names
                    ));
                }
                if units.len() == 0 {
                    return Err(format!("No unit found with name: {}", unit_name));
                }
                let x = units[0].id.clone();
                x
            };

            match crate::units::deactivate_unit(&id, run_info).map_err(|e| format!("{}", e)) {
                Err(e) => {
                    return Err(e);
                }
                Ok(_) => {
                    // Happy
                }
            };
        }
        Command::StopAll(unit_name) => {
            let run_info = &*run_info.read().unwrap();
            let id = {
                let units = find_units_with_name(&unit_name, &run_info.unit_table);
                if units.len() > 1 {
                    let names: Vec<_> = units.iter().map(|unit| unit.id.name.clone()).collect();
                    return Err(format!(
                        "More than one unit found with name: {}: {:?}",
                        unit_name, names
                    ));
                }
                if units.len() == 0 {
                    return Err(format!("No unit found with name: {}", unit_name));
                }
                let x = units[0].id.clone();
                x
            };

            match crate::units::deactivate_unit_recursive(&id, run_info)
                .map_err(|e| format!("{}", e))
            {
                Err(e) => {
                    return Err(e);
                }
                Ok(_) => {
                    // Happy
                }
            };
        }
        Command::Status(unit_name) => {
            let run_info = &*run_info.read().unwrap();
            let unit_table = &run_info.unit_table;
            match unit_name {
                Some(name) => {
                    //list specific
                    let units = find_units_with_pattern(&name, unit_table);
                    for unit in units {
                        let status = { unit.common.status.read().unwrap().clone() };
                        if name.ends_with(".service") {
                            result_vec
                                .as_array_mut()
                                .unwrap()
                                .push(format_service(&unit, status));
                        } else if name.ends_with(".socket") {
                            result_vec
                                .as_array_mut()
                                .unwrap()
                                .push(format_socket(&unit, status));
                        } else if name.ends_with(".target") {
                            result_vec
                                .as_array_mut()
                                .unwrap()
                                .push(format_target(&unit, status));
                        } else {
                            return Err("Name suffix not recognized".into());
                        }
                    }
                }
                None => {
                    //list all
                    let strings: Vec<_> = unit_table
                        .iter()
                        .map(|(_id, unit)| {
                            let status = { unit.common.status.read().unwrap().clone() };
                            match unit.specific {
                                Specific::Socket(_) => format_socket(&unit, status),
                                Specific::Service(_) => format_service(&unit, status),
                                Specific::Target(_) => format_target(&unit, status),
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
            let run_info = &*run_info.read().unwrap();
            let unit_table = &run_info.unit_table;
            for (id, unit) in unit_table.iter() {
                let include = if let Some(kind) = kind {
                    id.kind == kind
                } else {
                    true
                };
                if include {
                    result_vec
                        .as_array_mut()
                        .unwrap()
                        .push(Value::String(unit.id.name.clone()));
                }
            }
        }
        Command::LoadNew(names) => {
            let run_info = &mut *run_info.write().unwrap();
            let mut map = std::collections::HashMap::new();
            for name in &names {
                let unit = load_new_unit(&run_info.config.unit_dirs, &name)?;
                map.insert(unit.id.clone(), unit);
            }
            insert_new_units(map, run_info)?;
        }
        Command::LoadAllNew => {
            let run_info = &mut *run_info.write().unwrap();
            let unit_table = &run_info.unit_table;
            // get all units there are
            let units = load_all_units(&run_info.config.unit_dirs, &run_info.config.target_unit)
                .map_err(|e| format!("Error while loading unit definitons: {:?}", e))?;

            // collect all names
            let existing_names = unit_table
                .values()
                .map(|unit| unit.id.name.clone())
                .collect::<Vec<_>>();

            // filter out existing units
            let mut ignored_units_names = Vec::new();
            let mut new_units_names = Vec::new();
            let mut new_units = std::collections::HashMap::new();
            for (id, unit) in units {
                if existing_names.contains(&unit.id.name) {
                    ignored_units_names.push(Value::String(unit.id.name.clone()));
                } else {
                    new_units_names.push(Value::String(unit.id.name.clone()));
                    new_units.insert(id, unit);
                }
            }

            let mut response_object = serde_json::Map::new();
            insert_new_units(new_units, run_info)?;
            response_object.insert("Added".into(), serde_json::Value::Array(new_units_names));
            response_object.insert(
                "Ignored".into(),
                serde_json::Value::Array(ignored_units_names),
            );
            result_vec
                .as_array_mut()
                .unwrap()
                .push(Value::Object(response_object));
        }
        Command::LoadAllNewDry => {
            let run_info = &mut *run_info.write().unwrap();
            let unit_table = &run_info.unit_table;
            // get all units there are
            let units = load_all_units(&run_info.config.unit_dirs, &run_info.config.target_unit)
                .map_err(|e| format!("Error while loading unit definitons: {:?}", e))?;

            // collect all names
            let existing_names = unit_table
                .values()
                .map(|unit| unit.id.name.clone())
                .collect::<Vec<_>>();

            // filter out existing units
            let mut ignored_units_names = Vec::new();
            let mut new_units_names = Vec::new();
            let mut new_units = std::collections::HashMap::new();
            for (id, unit) in units {
                if existing_names.contains(&unit.id.name) {
                    ignored_units_names.push(Value::String(unit.id.name.clone()));
                } else {
                    new_units_names.push(Value::String(unit.id.name.clone()));
                    new_units.insert(id, unit);
                }
            }

            let mut response_object = serde_json::Map::new();
            response_object.insert(
                "Would add".into(),
                serde_json::Value::Array(new_units_names),
            );
            response_object.insert(
                "Would ignore".into(),
                serde_json::Value::Array(ignored_units_names),
            );
            result_vec
                .as_array_mut()
                .unwrap()
                .push(Value::Object(response_object));
        }
    }

    Ok(result_vec)
}

use std::io::Read;
use std::io::Write;
pub fn listen_on_commands<T: 'static + Read + Write + Send>(
    mut source: Box<T>,
    run_info: ArcMutRuntimeInfo,
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
                                let msg = match execute_command(cmd, run_info.clone()) {
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
    run_info: ArcMutRuntimeInfo,
    source: std::os::unix::net::UnixListener,
) {
    std::thread::spawn(move || loop {
        let stream = Box::new(source.accept().unwrap().0);
        listen_on_commands(stream, run_info.clone())
    });
}

pub fn accept_control_connections_tcp(run_info: ArcMutRuntimeInfo, source: std::net::TcpListener) {
    std::thread::spawn(move || loop {
        let stream = Box::new(source.accept().unwrap().0);
        listen_on_commands(stream, run_info.clone())
    });
}
