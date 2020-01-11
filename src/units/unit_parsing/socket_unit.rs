use crate::sockets::*;
use crate::units::*;
use std::path::PathBuf;

pub fn parse_socket(
    parsed_file: ParsedFile,
    path: &PathBuf,
    chosen_id: UnitId,
) -> Result<Unit, ParsingErrorReason> {
    let mut socket_configs = None;
    let mut install_config = None;
    let mut unit_config = None;
    let mut exec_config = None;

    for (name, mut section) in parsed_file {
        match name.as_str() {
            "[Socket]" => {
                exec_config = Some(super::parse_exec_section(&mut section)?);
                socket_configs = match parse_socket_section(section) {
                    Ok(conf) => Some(conf),
                    Err(e) => return Err(e),
                };
            }
            "[Unit]" => {
                unit_config = Some(parse_unit_section(section, path)?);
            }
            "[Install]" => {
                install_config = Some(parse_install_section(section)?);
            }

            _ => return Err(ParsingErrorReason::UnknownSection(name.to_owned())),
        }
    }

    let (sock_name, services, sock_configs) = match socket_configs {
        Some(triple) => triple,
        None => return Err(ParsingErrorReason::SectionNotFound("Socket".to_owned())),
    };

    let conf = match unit_config {
        Some(conf) => conf,
        None => return Err(ParsingErrorReason::SectionNotFound("Unit".to_owned())),
    };

    let exec_config = exec_config.unwrap();

    Ok(Unit {
        conf,
        id: chosen_id,
        install: Install {
            install_config: install_config,
            wants: Vec::new(),
            wanted_by: Vec::new(),
            requires: Vec::new(),
            required_by: Vec::new(),
            before: Vec::new(),
            after: Vec::new(),
        },
        specialized: UnitSpecialized::Socket(Socket {
            activated: false,
            name: sock_name,
            sockets: sock_configs,
            services,
            exec_config,
        }),
    })
}

fn parse_ipv4_addr(addr: &str) -> Result<std::net::SocketAddrV4, std::net::AddrParseError> {
    let sock: Result<std::net::SocketAddrV4, std::net::AddrParseError> = addr.parse();
    sock
}

fn parse_ipv6_addr(addr: &str) -> Result<std::net::SocketAddrV6, std::net::AddrParseError> {
    let sock: Result<std::net::SocketAddrV6, std::net::AddrParseError> = addr.parse();
    sock
}

fn parse_unix_addr(addr: &str) -> Result<String, ()> {
    if addr.starts_with('/') || addr.starts_with("./") {
        Ok(addr.to_owned())
    } else {
        Err(())
    }
}

fn parse_socket_section(
    mut section: ParsedSection,
) -> Result<(String, Vec<String>, Vec<SocketConfig>), ParsingErrorReason> {
    let fdname = section.remove("FILEDESCRIPTORNAME");
    let services = section.remove("SERVICE");
    let streams = section.remove("LISTENSTREAM");
    let datagrams = section.remove("LISTENDATAGRAM");
    let seqpacks = section.remove("LISTENSEQUENTIALPACKET");
    let fifos = section.remove("LISTENFIFO");

    if !section.is_empty() {
        return Err(ParsingErrorReason::UnusedSetting(
            section.keys().next().unwrap().to_owned(),
        ));
    }
    let fdname = match fdname {
        None => None,
        Some(mut vec) => {
            if vec.len() > 1 {
                return Err(ParsingErrorReason::SettingTooManyValues(
                    "FileDescriptorName".to_owned(),
                    super::map_tupels_to_second(vec),
                ));
            } else if vec.len() == 0 {
                None
            } else {
                Some(vec.remove(0).1)
            }
        }
    };

    let fdname = fdname.unwrap_or("unknown".into());

    let services = services
        .map(|vec| super::map_tupels_to_second(vec))
        .unwrap_or_default();

    let mut socket_kinds: Vec<(u32, SocketKind)> = Vec::new();
    if let Some(mut streams) = streams {
        for _ in 0..streams.len() {
            let (entry_num, value) = streams.remove(0);
            socket_kinds.push((entry_num, SocketKind::Stream(value)));
        }
    }
    if let Some(mut datagrams) = datagrams {
        for _ in 0..datagrams.len() {
            let (entry_num, value) = datagrams.remove(0);
            socket_kinds.push((entry_num, SocketKind::Datagram(value)));
        }
    }
    if let Some(mut seqpacks) = seqpacks {
        for _ in 0..seqpacks.len() {
            let (entry_num, value) = seqpacks.remove(0);
            socket_kinds.push((entry_num, SocketKind::Sequential(value)));
        }
    }
    if let Some(mut fifos) = fifos {
        for _ in 0..fifos.len() {
            let (entry_num, value) = fifos.remove(0);
            socket_kinds.push((entry_num, SocketKind::Fifo(value)));
        }
    }

    // we need to preserve the original ordering
    socket_kinds.sort_by(|l, r| u32::cmp(&l.0, &r.0));
    let socket_kinds: Vec<SocketKind> = socket_kinds.drain(..).map(|(_, kind)| kind).collect();

    let mut socket_configs = Vec::new();

    for kind in socket_kinds {
        let specialized: SpecializedSocketConfig = match &kind {
            SocketKind::Fifo(addr) => {
                if parse_unix_addr(addr).is_ok() {
                    SpecializedSocketConfig::Fifo(FifoConfig {
                        path: std::path::PathBuf::from(addr),
                    })
                } else {
                    return Err(ParsingErrorReason::UnknownSocketAddr(addr.to_owned()));
                }
            }
            SocketKind::Sequential(addr) => {
                if parse_unix_addr(addr).is_ok() {
                    SpecializedSocketConfig::UnixSocket(UnixSocketConfig::Sequential(addr.clone()))
                } else {
                    return Err(ParsingErrorReason::UnknownSocketAddr(addr.to_owned()));
                }
            }
            SocketKind::Stream(addr) => {
                if parse_unix_addr(addr).is_ok() {
                    SpecializedSocketConfig::UnixSocket(UnixSocketConfig::Stream(addr.clone()))
                } else if let Ok(addr) = parse_ipv4_addr(addr) {
                    SpecializedSocketConfig::TcpSocket(TcpSocketConfig {
                        addr: std::net::SocketAddr::V4(addr),
                    })
                } else if let Ok(addr) = parse_ipv6_addr(addr) {
                    SpecializedSocketConfig::TcpSocket(TcpSocketConfig {
                        addr: std::net::SocketAddr::V6(addr),
                    })
                } else {
                    return Err(ParsingErrorReason::UnknownSocketAddr(addr.to_owned()));
                }
            }
            SocketKind::Datagram(addr) => {
                if parse_unix_addr(addr).is_ok() {
                    SpecializedSocketConfig::UnixSocket(UnixSocketConfig::Datagram(addr.clone()))
                } else if let Ok(addr) = parse_ipv4_addr(addr) {
                    SpecializedSocketConfig::UdpSocket(UdpSocketConfig {
                        addr: std::net::SocketAddr::V4(addr),
                    })
                } else if let Ok(addr) = parse_ipv6_addr(addr) {
                    SpecializedSocketConfig::UdpSocket(UdpSocketConfig {
                        addr: std::net::SocketAddr::V6(addr),
                    })
                } else {
                    return Err(ParsingErrorReason::UnknownSocketAddr(addr.to_owned()));
                }
            }
        };

        socket_configs.push(SocketConfig { kind, specialized });
    }

    Ok((fdname, services, socket_configs))
}
