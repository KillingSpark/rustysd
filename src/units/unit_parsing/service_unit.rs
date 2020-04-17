use crate::units::*;
use std::path::PathBuf;

pub fn parse_service(
    parsed_file: ParsedFile,
    path: &PathBuf,
) -> Result<ParsedServiceConfig, ParsingErrorReason> {
    let mut service_config = None;
    let mut install_config = None;
    let mut unit_config = None;

    for (name, section) in parsed_file {
        match name.as_str() {
            "[Service]" => {
                service_config = Some(parse_service_section(section)?);
            }
            "[Unit]" => {
                unit_config = Some(parse_unit_section(section)?);
            }
            "[Install]" => {
                install_config = Some(parse_install_section(section)?);
            }

            _ => return Err(ParsingErrorReason::UnknownSection(name.to_owned())),
        }
    }

    let service_config = if let Some(service_config) = service_config {
        service_config
    } else {
        return Err(ParsingErrorReason::SectionNotFound("Service".to_owned()));
    };

    Ok(ParsedServiceConfig {
        common: ParsedCommonConfig {
            name: path.file_name().unwrap().to_str().unwrap().to_owned(),
            unit: unit_config.unwrap_or_else(Default::default),
            install: install_config.unwrap_or_else(Default::default),
        },
        srvc: service_config,
    })
}

fn parse_timeout(descr: &str) -> Timeout {
    if descr.to_uppercase() == "INFINITY" {
        Timeout::Infinity
    } else {
        match descr.parse::<u64>() {
            Ok(secs) => Timeout::Duration(std::time::Duration::from_secs(secs)),
            Err(_) => {
                let mut sum = 0;
                let split = descr.split(' ').collect::<Vec<_>>();
                for t in &split {
                    if t.ends_with("min") {
                        let mins = t[0..t.len() - 3].parse::<u64>().unwrap();
                        sum += mins * 60;
                    } else if t.ends_with("hrs") {
                        let hrs = t[0..t.len() - 3].parse::<u64>().unwrap();
                        sum += hrs * 60 * 60;
                    } else if t.ends_with("s") {
                        let secs = t[0..t.len() - 1].parse::<u64>().unwrap();
                        sum += secs;
                    }
                }
                Timeout::Duration(std::time::Duration::from_secs(sum))
            }
        }
    }
}

fn parse_cmdlines(raw_lines: &Vec<(u32, String)>) -> Result<Vec<Commandline>, ParsingErrorReason> {
    let mut cmdlines = Vec::new();
    for (_line, cmdline) in raw_lines {
        cmdlines.push(parse_cmdline(cmdline)?);
    }
    Ok(cmdlines)
}

fn parse_cmdline(raw_line: &str) -> Result<Commandline, ParsingErrorReason> {
    let mut split = shlex::split(raw_line).ok_or(ParsingErrorReason::Generic(format!(
        "Could not parse cmdline: {}",
        raw_line
    )))?;
    let mut cmd = split.remove(0);

    let mut prefixes = Vec::new();
    loop {
        let prefix = match &cmd[..1] {
            "-" => {
                cmd = cmd[1..].to_owned();
                CommandlinePrefix::Minus
            }
            "+" => {
                return Err(ParsingErrorReason::UnsupportedSetting(
                    "The prefix '+' for cmdlines is currently not supported".into(),
                ));
                //cmd = cmd[1..].to_owned();
                //CommandlinePrefix::Plus
            }
            "@" => {
                return Err(ParsingErrorReason::UnsupportedSetting(
                    "The prefix '@' for cmdlines is currently not supported".into(),
                ));
                //cmd = cmd[1..].to_owned();
                //CommandlinePrefix::AtSign
            }
            ":" => {
                return Err(ParsingErrorReason::UnsupportedSetting(
                    "The prefix ':' for cmdlines is currently not supported".into(),
                ));
                //cmd = cmd[1..].to_owned();
                //CommandlinePrefix::Colon
            }
            "!" => match &cmd[1..2] {
                "!" => {
                    return Err(ParsingErrorReason::UnsupportedSetting(
                        "The prefix '!!' for cmdlines is currently not supported".into(),
                    ));
                    //cmd = cmd[2..].to_owned();
                    //CommandlinePrefix::DoubleExclamation
                }
                _ => {
                    return Err(ParsingErrorReason::UnsupportedSetting(
                        "The prefix '!' for cmdlines is currently not supported".into(),
                    ));
                    //cmd = cmd[1..].to_owned();
                    //CommandlinePrefix::Exclamation
                }
            },
            _ => break,
        };
        prefixes.push(prefix);
    }
    Ok(Commandline {
        cmd,
        prefixes,
        args: split,
    })
}

fn parse_service_section(
    mut section: ParsedSection,
) -> Result<ParsedServiceSection, ParsingErrorReason> {
    let exec = section.remove("EXECSTART");
    let stop = section.remove("EXECSTOP");
    let stoppost = section.remove("EXECSTOPPOST");
    let startpre = section.remove("EXECSTARTPRE");
    let startpost = section.remove("EXECSTARTPOST");
    let starttimeout = section.remove("TIMEOUTSTARTSEC");
    let stoptimeout = section.remove("TIMEOUTSTOPSEC");
    let generaltimeout = section.remove("TIMEOUTSEC");

    let restart = section.remove("RESTART");
    let sockets = section.remove("SOCKETS");
    let notify_access = section.remove("NOTIFYACCESS");
    let srcv_type = section.remove("TYPE");
    let accept = section.remove("ACCEPT");
    let dbus_name = section.remove("BUSNAME");

    let exec_config = super::parse_exec_section(&mut section)?;

    if !section.is_empty() {
        return Err(ParsingErrorReason::UnusedSetting(
            section.keys().next().unwrap().to_owned(),
        ));
    }

    let starttimeout = match starttimeout {
        Some(vec) => {
            if vec.len() == 1 {
                Some(parse_timeout(&vec[0].1))
            } else {
                return Err(ParsingErrorReason::SettingTooManyValues(
                    "TimeoutStartSec".to_owned(),
                    super::map_tupels_to_second(vec),
                ));
            }
        }
        None => None,
    };
    let stoptimeout = match stoptimeout {
        Some(vec) => {
            if vec.len() == 1 {
                Some(parse_timeout(&vec[0].1))
            } else {
                return Err(ParsingErrorReason::SettingTooManyValues(
                    "TimeoutStopSec".to_owned(),
                    super::map_tupels_to_second(vec),
                ));
            }
        }
        None => None,
    };
    let generaltimeout = match generaltimeout {
        Some(vec) => {
            if vec.len() == 1 {
                Some(parse_timeout(&vec[0].1))
            } else {
                return Err(ParsingErrorReason::SettingTooManyValues(
                    "TimeoutSec".to_owned(),
                    super::map_tupels_to_second(vec),
                ));
            }
        }
        None => None,
    };

    let exec = match exec {
        Some(mut vec) => {
            if vec.len() == 1 {
                parse_cmdline(&vec.remove(0).1)?
            } else {
                return Err(ParsingErrorReason::SettingTooManyValues(
                    "ExecStart".to_owned(),
                    super::map_tupels_to_second(vec),
                ));
            }
        }
        None => return Err(ParsingErrorReason::MissingSetting("ExecStart".to_owned())),
    };

    let srcv_type = match srcv_type {
        Some(vec) => {
            if vec.len() == 1 {
                match vec[0].1.as_str() {
                    "simple" => ServiceType::Simple,
                    "notify" => ServiceType::Notify,
                    "oneshot" => ServiceType::OneShot,
                    "dbus" => {
                        if cfg!(feature = "dbus_support") {
                            ServiceType::Dbus
                        } else {
                            return Err(ParsingErrorReason::UnsupportedSetting(
                                "Type=dbus".to_owned(),
                            ));
                        }
                    }
                    name => {
                        return Err(ParsingErrorReason::UnknownSetting(
                            "Type".to_owned(),
                            name.to_owned(),
                        ))
                    }
                }
            } else if vec.len() == 0 {
                return Err(ParsingErrorReason::MissingSetting("Type".to_owned()));
            } else {
                return Err(ParsingErrorReason::SettingTooManyValues(
                    "Type".to_owned(),
                    super::map_tupels_to_second(vec),
                ));
            }
        }
        None => ServiceType::Simple,
    };

    let notifyaccess = match notify_access {
        Some(vec) => {
            if vec.len() == 1 {
                match vec[0].1.as_str() {
                    "all" => NotifyKind::All,
                    "main" => NotifyKind::Main,
                    "exec" => NotifyKind::Exec,
                    "none" => NotifyKind::None,
                    name => {
                        return Err(ParsingErrorReason::UnknownSetting(
                            "NotifyAccess".to_owned(),
                            name.to_owned(),
                        ))
                    }
                }
            } else {
                return Err(ParsingErrorReason::SettingTooManyValues(
                    "NotifyAccess".to_owned(),
                    super::map_tupels_to_second(vec),
                ));
            }
        }
        None => NotifyKind::Main,
    };

    let stop = match stop {
        Some(vec) => parse_cmdlines(&vec)?,
        None => Vec::new(),
    };
    let stoppost = match stoppost {
        Some(vec) => parse_cmdlines(&vec)?,
        None => Vec::new(),
    };
    let startpre = match startpre {
        Some(vec) => parse_cmdlines(&vec)?,
        None => Vec::new(),
    };
    let startpost = match startpost {
        Some(vec) => parse_cmdlines(&vec)?,
        None => Vec::new(),
    };

    let restart = match restart {
        Some(vec) => {
            if vec.len() == 1 {
                match vec[0].1.to_uppercase().as_str() {
                    "ALWAYS" => ServiceRestart::Always,
                    "NO" => ServiceRestart::No,

                    name => {
                        return Err(ParsingErrorReason::UnknownSetting(
                            "Restart".to_owned(),
                            name.to_owned(),
                        ))
                    }
                }
            } else {
                return Err(ParsingErrorReason::SettingTooManyValues(
                    "Restart".to_owned(),
                    super::map_tupels_to_second(vec),
                ));
            }
        }
        None => ServiceRestart::No,
    };
    let accept = match accept {
        Some(vec) => {
            if vec.len() == 1 {
                string_to_bool(&vec[0].1)
            } else {
                return Err(ParsingErrorReason::SettingTooManyValues(
                    "Accept".to_owned(),
                    super::map_tupels_to_second(vec),
                ));
            }
        }
        None => false,
    };
    let dbus_name = match dbus_name {
        Some(vec) => {
            if vec.len() == 1 {
                Some(vec[0].1.to_owned())
            } else {
                return Err(ParsingErrorReason::SettingTooManyValues(
                    "BusName".to_owned(),
                    super::map_tupels_to_second(vec),
                ));
            }
        }
        None => None,
    };

    if let ServiceType::Dbus = srcv_type {
        if dbus_name.is_none() {
            return Err(ParsingErrorReason::MissingSetting("BusName".to_owned()));
        }
    }

    Ok(ParsedServiceSection {
        srcv_type,
        notifyaccess,
        restart,
        accept,
        dbus_name,
        exec,
        stop,
        stoppost,
        startpre,
        startpost,
        starttimeout,
        stoptimeout,
        generaltimeout,
        sockets: map_tupels_to_second(sockets.unwrap_or_default()),
        exec_section: exec_config,
    })
}
