use crate::services::*;
use crate::sockets::*;
use crate::units::*;

#[cfg(feature = "cgroups")]
use log::trace;

use std::convert::TryInto;
use std::path::PathBuf;
use std::sync::RwLock;

#[cfg(feature = "cgroups")]
fn make_cgroup_path(srvc_name: &str) -> Result<PathBuf, String> {
    let rustysd_cgroup =
        crate::platform::cgroups::get_own_freezer(&PathBuf::from("/sys/fs/cgroup"))
            .map_err(|e| format!("Couldnt get own cgroup: {}", e))?;
    let service_cgroup = rustysd_cgroup.join(srvc_name);
    trace!(
        "Service {} will be moved into cgroup: {:?}",
        srvc_name,
        service_cgroup
    );
    Ok(service_cgroup)
}

#[cfg(not(feature = "cgroups"))]
fn make_cgroup_path(_srvc_name: &str) -> Result<PathBuf, String> {
    // doesnt matter, wont be used anyways
    Ok(PathBuf::from("/ree"))
}

pub fn unit_from_parsed_service(conf: ParsedServiceConfig) -> Result<Unit, String> {
    // TODO make the cgroup path dynamic so multiple rustysd instances can exist
    let platform_specific = PlatformSpecificServiceFields {
        #[cfg(target_os = "linux")]
        cgroup_path: make_cgroup_path(&conf.common.name)?,
    };

    let mut sockets: Vec<UnitId> = Vec::new();
    for sock in conf.srvc.sockets {
        sockets.push(sock.as_str().try_into()?);
    }

    let mut common = make_common_from_parsed(conf.common.unit, conf.common.install)?;
    common.unit.refs_by_name.extend(sockets.iter().cloned());

    Ok(Unit {
        id: UnitId {
            kind: UnitIdKind::Service,
            name: conf.common.name,
        },
        common,
        specific: Specific::Service(ServiceSpecific {
            conf: ServiceConfig {
                exec_config: conf.srvc.exec_section.try_into()?,
                sockets: sockets,
                accept: conf.srvc.accept,
                dbus_name: conf.srvc.dbus_name,
                restart: conf.srvc.restart,
                notifyaccess: conf.srvc.notifyaccess,
                exec: conf.srvc.exec,
                startpre: conf.srvc.startpre,
                startpost: conf.srvc.startpost,
                stop: conf.srvc.stop,
                stoppost: conf.srvc.stoppost,
                srcv_type: conf.srvc.srcv_type,
                starttimeout: conf.srvc.starttimeout,
                stoptimeout: conf.srvc.stoptimeout,
                generaltimeout: conf.srvc.generaltimeout,
                platform_specific,
            },
            state: RwLock::new(ServiceState {
                common: CommonState::default(),
                srvc: Service {
                    pid: None,
                    status_msgs: Vec::new(),
                    process_group: None,
                    signaled_ready: false,
                    notifications: None,
                    notifications_path: None,
                    stdout: None,
                    stderr: None,
                    notifications_buffer: String::new(),
                    stdout_buffer: Vec::new(),
                    stderr_buffer: Vec::new(),
                },
            }),
        }),
    })
}

pub fn unit_from_parsed_socket(conf: ParsedSocketConfig) -> Result<Unit, String> {
    let mut services: Vec<UnitId> = Vec::new();
    for srvc in conf.sock.services {
        services.push(srvc.as_str().try_into()?);
    }

    let mut common = make_common_from_parsed(conf.common.unit, conf.common.install)?;
    common.unit.refs_by_name.extend(services.iter().cloned());

    Ok(Unit {
        id: UnitId {
            kind: UnitIdKind::Socket,
            name: conf.common.name,
        },
        common,
        specific: Specific::Socket(SocketSpecific {
            conf: SocketConfig {
                exec_config: conf.sock.exec_section.try_into()?,
                filedesc_name: conf.sock.filedesc_name.unwrap_or("unknown".to_owned()),
                services: services,
                sockets: conf.sock.sockets.into_iter().map(Into::into).collect(),
            },
            state: RwLock::new(SocketState {
                common: CommonState::default(),
                sock: Socket { activated: false },
            }),
        }),
    })
}
pub fn unit_from_parsed_target(conf: ParsedTargetConfig) -> Result<Unit, String> {
    Ok(Unit {
        id: UnitId {
            kind: UnitIdKind::Target,
            name: conf.common.name,
        },
        common: make_common_from_parsed(conf.common.unit, conf.common.install)?,
        specific: Specific::Target(TargetSpecific {
            state: RwLock::new(TargetState {
                common: CommonState::default(),
            }),
        }),
    })
}

impl From<ParsedSingleSocketConfig> for SingleSocketConfig {
    fn from(parsed: ParsedSingleSocketConfig) -> SingleSocketConfig {
        SingleSocketConfig {
            kind: parsed.kind,
            specialized: parsed.specialized,
        }
    }
}

impl std::convert::TryFrom<ParsedExecSection> for ExecConfig {
    type Error = String;
    fn try_from(parsed: ParsedExecSection) -> Result<ExecConfig, String> {
        let uid = if let Some(user) = &parsed.user {
            if let Ok(uid) = user.parse::<u32>() {
                Some(nix::unistd::Uid::from_raw(uid))
            } else {
                if let Ok(pwentry) = crate::platform::pwnam::getpwnam_r(&user)
                    .map_err(|e| ParsingErrorReason::Generic(e))
                {
                    Some(pwentry.uid)
                } else {
                    return Err(format!("Couldnt get uid for username: {}", user));
                }
            }
        } else {
            None
        };
        let uid = uid.unwrap_or(nix::unistd::getuid());

        let gid = if let Some(group) = &parsed.group {
            if let Ok(gid) = group.parse::<u32>() {
                Some(nix::unistd::Gid::from_raw(gid))
            } else {
                if let Ok(groupentry) = crate::platform::grnam::getgrnam_r(&group)
                    .map_err(|e| ParsingErrorReason::Generic(e))
                {
                    Some(groupentry.gid)
                } else {
                    return Err(format!("Couldnt get gid for groupname: {}", group));
                }
            }
        } else {
            None
        };
        let gid = gid.unwrap_or(nix::unistd::getgid());

        let mut supp_gids = Vec::new();
        for group in &parsed.supplementary_groups {
            let gid = if let Ok(gid) = group.parse::<u32>() {
                nix::unistd::Gid::from_raw(gid)
            } else {
                if let Ok(groupentry) = crate::platform::grnam::getgrnam_r(&group)
                    .map_err(|e| ParsingErrorReason::Generic(e))
                {
                    groupentry.gid
                } else {
                    return Err(format!("Couldnt get gid for groupname: {}", group));
                }
            };
            supp_gids.push(gid);
        }
        Ok(ExecConfig {
            user: uid,
            group: gid,
            supplementary_groups: supp_gids,
            stderr_path: parsed.stderr_path,
            stdout_path: parsed.stdout_path,
            environment: parsed.environment,
        })
    }
}

fn make_common_from_parsed(
    unit: ParsedUnitSection,
    install: ParsedInstallSection,
) -> Result<Common, String> {
    let mut wants = Vec::new();
    for name in unit.wants {
        wants.push(name.as_str().try_into()?);
    }
    let mut requires = Vec::new();
    for name in unit.requires {
        requires.push(name.as_str().try_into()?);
    }
    let mut wanted_by = Vec::new();
    for name in install.wanted_by {
        wanted_by.push(name.as_str().try_into()?);
    }
    let mut required_by = Vec::new();
    for name in install.required_by {
        required_by.push(name.as_str().try_into()?);
    }
    let mut after = Vec::new();
    for name in unit.after {
        after.push(name.as_str().try_into()?);
    }
    let mut before = Vec::new();
    for name in unit.before {
        before.push(name.as_str().try_into()?);
    }

    let mut refs_by_name = Vec::new();
    refs_by_name.extend(wants.iter().cloned());
    refs_by_name.extend(wanted_by.iter().cloned());
    refs_by_name.extend(requires.iter().cloned());
    refs_by_name.extend(required_by.iter().cloned());
    refs_by_name.extend(before.iter().cloned());
    refs_by_name.extend(after.iter().cloned());

    Ok(Common {
        status: RwLock::new(UnitStatus::NeverStarted),
        unit: UnitConfig {
            description: unit.description,
            refs_by_name,
        },
        dependencies: Dependencies {
            wants,
            wanted_by,
            requires,
            required_by,
            after,
            before,
        },
    })
}

impl std::convert::TryInto<UnitId> for &str {
    type Error = String;
    fn try_into(self) -> Result<UnitId, String> {
        if self.ends_with(".target") {
            Ok(UnitId {
                name: self.to_owned(),
                kind: UnitIdKind::Target,
            })
        } else if self.ends_with(".service") {
            Ok(UnitId {
                name: self.to_owned(),
                kind: UnitIdKind::Service,
            })
        } else if self.ends_with(".socket") {
            Ok(UnitId {
                name: self.to_owned(),
                kind: UnitIdKind::Socket,
            })
        } else {
            Err(format!(
                "{} is not a valid unit name. The suffix is not supported.",
                self
            ))
        }
    }
}

impl std::convert::TryFrom<ParsedServiceConfig> for Unit {
    type Error = String;
    fn try_from(conf: ParsedServiceConfig) -> Result<Unit, String> {
        unit_from_parsed_service(conf)
    }
}
impl std::convert::TryFrom<ParsedSocketConfig> for Unit {
    type Error = String;
    fn try_from(conf: ParsedSocketConfig) -> Result<Unit, String> {
        unit_from_parsed_socket(conf)
    }
}
impl std::convert::TryFrom<ParsedTargetConfig> for Unit {
    type Error = String;
    fn try_from(conf: ParsedTargetConfig) -> Result<Unit, String> {
        unit_from_parsed_target(conf)
    }
}
