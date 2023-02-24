//! Config can be loaded either from env vars, toml, or json.
//!
//! Currently configurable:
//! ### Logging
//! 1. Wether or not to log to disk (and the dir to put the logs in)
//! 1. Wether or not to log to stdout
//!
//! ### General config
//! 1. Where to find the units (one or more directories)
//! 1. notification-socket directory (where the unix-domain sockets are placed on which services can notify rustysd)
//! 1. Which unit is the target that should be started

use std::{collections::HashMap, fs::File, io::Read, path::PathBuf};
use toml;

#[derive(Debug)]
pub struct LoggingConfig {
    pub log_to_stdout: bool,
    pub log_to_disk: bool,
    pub log_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub unit_dirs: Vec<PathBuf>,
    pub target_unit: String,
    pub notification_sockets_dir: PathBuf,
    pub self_path: PathBuf,
}

#[derive(Debug)]
enum SettingValue {
    Str(String),
    Array(Vec<SettingValue>),
    Boolean(bool),
}

fn load_toml(
    config_path: &PathBuf,
    settings: &mut HashMap<String, SettingValue>,
) -> Result<(), String> {
    let mut file =
        File::open(config_path).map_err(|e| format!("Error while opening config file: {}", e))?;
    let mut config = String::new();
    file.read_to_string(&mut config).unwrap();

    let toml_conf =
        toml::from_str(&config).map_err(|e| format!("Error while decoding config toml: {}", e))?;

    if let toml::Value::Table(map) = &toml_conf {
        if let Some(toml::Value::Array(elems)) = map.get("unit_dirs") {
            settings.insert(
                "unit.dirs".to_owned(),
                SettingValue::Array(
                    elems
                        .iter()
                        .map(|e| {
                            if let toml::Value::String(s) = e {
                                SettingValue::Str(s.clone())
                            } else {
                                SettingValue::Str("".to_owned())
                            }
                        })
                        .collect(),
                ),
            );
        }

        if let Some(toml::Value::String(val)) = map.get("logging_dir") {
            settings.insert("logging.dir".to_owned(), SettingValue::Str(val.clone()));
        }
        if let Some(toml::Value::Boolean(val)) = map.get("log_to_disk") {
            settings.insert("logging.to.disk".to_owned(), SettingValue::Boolean(*val));
        }
        if let Some(toml::Value::Boolean(val)) = map.get("log_to_stdout") {
            settings.insert("logging.to.stdout".to_owned(), SettingValue::Boolean(*val));
        }
        if let Some(toml::Value::String(val)) = map.get("target_unit") {
            settings.insert("target.unit".to_owned(), SettingValue::Str(val.clone()));
        }
        if let Some(toml::Value::String(val)) = map.get("selfpath") {
            settings.insert("selfpath".to_owned(), SettingValue::Str(val.clone()));
        }
        if let Some(toml::Value::String(val)) = map.get("notifications_dir") {
            settings.insert(
                "notifications.dir".to_owned(),
                SettingValue::Str(val.clone()),
            );
        }
    }
    Ok(())
}

fn load_json(
    config_path: &PathBuf,
    settings: &mut HashMap<String, SettingValue>,
) -> Result<(), String> {
    let mut file =
        File::open(config_path).map_err(|e| format!("Error while decoding config json: {}", e))?;
    let json_conf = serde_json::from_reader(&mut file)
        .map_err(|e| format!("Error while decoding config json: {}", e))?;

    if let serde_json::Value::Object(map) = &json_conf {
        if let Some(serde_json::Value::Array(elems)) = map.get("unit_dirs") {
            settings.insert(
                "unit.dirs".to_owned(),
                SettingValue::Array(
                    elems
                        .iter()
                        .map(|e| {
                            if let serde_json::Value::String(s) = e {
                                SettingValue::Str(s.clone())
                            } else {
                                SettingValue::Str("".to_owned())
                            }
                        })
                        .collect(),
                ),
            );
        }

        if let Some(serde_json::Value::String(val)) = map.get("logging_dir") {
            settings.insert("logging.dir".to_owned(), SettingValue::Str(val.clone()));
        }
        if let Some(serde_json::Value::Bool(val)) = map.get("log_to_disk") {
            settings.insert("logging.to_disk".to_owned(), SettingValue::Boolean(*val));
        }
        if let Some(serde_json::Value::Bool(val)) = map.get("log_to_stdout") {
            settings.insert("logging.to_stdout".to_owned(), SettingValue::Boolean(*val));
        }
        if let Some(serde_json::Value::String(val)) = map.get("target_unit") {
            settings.insert("target.unit".to_owned(), SettingValue::Str(val.clone()));
        }
        if let Some(serde_json::Value::String(val)) = map.get("selfpath") {
            settings.insert("selfpath".to_owned(), SettingValue::Str(val.clone()));
        }
        if let Some(serde_json::Value::String(val)) = map.get("notifications_dir") {
            settings.insert(
                "notifications.dir".to_owned(),
                SettingValue::Str(val.clone()),
            );
        }
    }
    Ok(())
}

pub fn load_config(config_path: &Option<PathBuf>) -> (LoggingConfig, Result<Config, String>) {
    let mut settings: HashMap<String, SettingValue> = HashMap::new();

    let default_config_path_json = PathBuf::from("./config/rustysd_config.json");
    let default_config_path_toml = PathBuf::from("./config/rustysd_config.toml");

    let config_path_json = if let Some(config_path) = config_path {
        config_path.join("rustysd_config.json")
    } else {
        default_config_path_json
    };

    let config_path_toml = if let Some(config_path) = config_path {
        config_path.join("rustysd_config.toml")
    } else {
        default_config_path_toml.clone()
    };

    let json_conf = if config_path_json.exists() {
        Some(load_json(&config_path_json, &mut settings))
    } else {
        None
    };

    let toml_conf = if config_path_toml.exists() {
        Some(load_toml(&config_path_toml, &mut settings))
    } else {
        None
    };

    std::env::vars().for_each(|(key, value)| {
        let mut new_key: Vec<String> = key.split('_').map(|part| part.to_lowercase()).collect();
        //drop prefix
        if *new_key[0] == *"rustysd" {
            new_key.remove(0);
            let new_key = new_key.join(".");
            settings.insert(new_key, SettingValue::Str(value));
        }
    });

    let log_dir = settings.get("logging.dir").map(|dir| match dir {
        SettingValue::Str(s) => Some(PathBuf::from(s)),
        _ => None,
    });

    let log_to_stdout = settings.get("logging.to_stdout").map(|val| match val {
        SettingValue::Boolean(b) => *b,
        _ => false,
    });
    let log_to_disk = settings.get("logging.to_disk").map(|val| match val {
        SettingValue::Boolean(b) => *b,
        _ => false,
    });

    let notification_sockets_dir = settings.get("notifications.dir").and_then(|dir| match dir {
        SettingValue::Str(s) => Some(PathBuf::from(s)),
        _ => None,
    });
    let target_unit = settings.get("target.unit").and_then(|name| match name {
        SettingValue::Str(s) => Some(s.clone()),
        _ => None,
    });
    let self_path = settings.get("selfpath").and_then(|dir| match dir {
        SettingValue::Str(s) => Some(PathBuf::from(s)),
        _ => None,
    });

    let unit_dirs = settings.get("unit.dirs").map(|dir| match dir {
        SettingValue::Str(s) => vec![PathBuf::from(s)],
        SettingValue::Array(arr) => arr
            .iter()
            .map(|el| match el {
                SettingValue::Str(s) => Some(PathBuf::from(s)),
                _ => None,
            })
            .fold(Vec::new(), |mut acc, el| {
                if let Some(path) = el {
                    if path.exists() {
                        acc.push(path)
                    }
                }
                acc
            }),
        _ => Vec::new(),
    });

    let config = Config {
        unit_dirs: unit_dirs.unwrap_or_else(|| vec![PathBuf::from("./unitfiles")]),
        target_unit: target_unit.unwrap_or("default.target".to_owned()),

        notification_sockets_dir: notification_sockets_dir
            .unwrap_or_else(|| PathBuf::from("./notifications")),

        self_path: self_path.unwrap_or_else(|| {
            std::env::current_exe()
                .expect("Could not get own executable name and it was not configured explicitly")
        }),
    };

    let conf = if let Some(json_conf) = json_conf {
        if toml_conf.is_some() {
            Err(format!("Found both json and toml conf!"))
        } else {
            match json_conf {
                Err(e) => Err(e),
                Ok(_) => Ok(config),
            }
        }
    } else {
        match toml_conf {
            Some(Err(e)) => Err(e),
            Some(Ok(_)) => Ok(config),
            None => {
                if *config_path_toml == default_config_path_toml {
                    Ok(config)
                } else {
                    Err("No config file was loaded".into())
                }
            }
        }
    };

    (
        LoggingConfig {
            log_dir: log_dir
                .unwrap_or_else(|| Some(PathBuf::from("./logs")))
                .unwrap_or_else(|| PathBuf::from("./logs")),
            log_to_disk: log_to_disk.unwrap_or(false),
            log_to_stdout: log_to_stdout.unwrap_or(true),
        },
        conf,
    )
}
