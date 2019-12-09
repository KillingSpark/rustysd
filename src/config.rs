use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;

pub struct LoggingConfig {
    pub log_dir: PathBuf,
}

pub struct Config {
    pub unit_dirs: Vec<PathBuf>,
    pub notification_sockets_dir: PathBuf,
}

pub fn load_config(config_path: Option<&PathBuf>) -> (LoggingConfig, Result<Config, String>) {
    let mut settings = HashMap::new();

    let default_config_path = PathBuf::from("./config/rustysd_config.json");
    let config_path = config_path.unwrap_or(&default_config_path);
    let json_conf = if config_path.exists() {
        match File::open(config_path) {
            Ok(mut file) => Some(
                serde_json::from_reader(&mut file)
                    .map_err(|e| format!("Error while decoding config json: {}", e)),
            ),
            Err(e) => Some(Err(format!("Error while opening config file: {}", e))),
        }
    } else {
        None
    };
    if let Some(Ok(json_conf)) = &json_conf {
        parse_all_settings_json(&json_conf, "", &mut settings, false);
    }

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

    let notification_sockets_dir = settings.get("notifications.dir").map(|dir| match dir {
        SettingValue::Str(s) => Some(PathBuf::from(s)),
        _ => None,
    });

    let unit_dirs = settings.get("notifications.dir").map(|dir| match dir {
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
        unit_dirs: unit_dirs.unwrap_or_else(|| vec![PathBuf::from("./test_units")]),

        notification_sockets_dir: notification_sockets_dir
            .unwrap_or_else(|| Some(PathBuf::from("./notifications")))
            .unwrap_or_else(|| PathBuf::from("./notifications")),
    };

    let conf = match json_conf {
        Some(Err(e)) => Err(e),
        Some(Ok(_)) => Ok(config),
        None => {
            if *config_path == default_config_path {
                Ok(config)
            } else {
                Err("Json config file not loaded".into())
            }
        }
    };

    (
        LoggingConfig {
            log_dir: log_dir
                .unwrap_or_else(|| Some(PathBuf::from("./logs")))
                .unwrap_or_else(|| PathBuf::from("./logs")),
        },
        conf,
    )
}

enum SettingValue {
    Str(String),
    Bool(bool),
    Num(serde_json::Number),
    Array(Vec<SettingValue>),
}

fn parse_all_settings_json(
    json: &serde_json::Value,
    prefix: &str,
    existing_settings: &mut HashMap<String, SettingValue>,
    override_existing: bool,
) {
    match json {
        serde_json::Value::Object(map) => {
            for (name, value) in map {
                let mut obj_name = String::from(prefix);
                obj_name.push('.');
                obj_name.push_str(&name);
                parse_all_settings_json(value, &obj_name, existing_settings, override_existing);
            }
        }
        serde_json::Value::Array(elems) => {
            if !existing_settings.contains_key(prefix) || override_existing {
                let arr = SettingValue::Array(
                    elems
                        .iter()
                        .map(|el| match el {
                            serde_json::Value::Bool(b) => SettingValue::Bool(*b),
                            serde_json::Value::String(s) => SettingValue::Str(s.clone()),
                            serde_json::Value::Number(n) => SettingValue::Num(n.clone()),
                            other => SettingValue::Str(other.to_string()),
                        })
                        .collect(),
                );
                existing_settings.insert(prefix.to_owned(), arr);
            }
        }
        serde_json::Value::String(value) => {
            if !existing_settings.contains_key(prefix) || override_existing {
                existing_settings.insert(prefix.to_owned(), SettingValue::Str(value.clone()));
            }
        }
        serde_json::Value::Number(value) => {
            if !existing_settings.contains_key(prefix) || override_existing {
                existing_settings.insert(prefix.to_owned(), SettingValue::Num(value.clone()));
            }
        }
        serde_json::Value::Bool(value) => {
            if !existing_settings.contains_key(prefix) || override_existing {
                existing_settings.insert(prefix.to_owned(), SettingValue::Bool(*value));
            }
        }
        serde_json::Value::Null => {
            //ignore
        }
    }
}
