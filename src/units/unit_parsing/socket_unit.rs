use crate::sockets::*;
use crate::units::*;
use std::path::PathBuf;


pub fn parse_socket(parsed_file: ParsedFile, path: &PathBuf, chosen_id: InternalId) -> Result<Unit, String> {
    let mut socket_configs = None;
    let mut install_config = None;
    let mut unit_config = None;

    for (name, section) in parsed_file {
        match name.as_str() {
            "[Socket]" => {
                socket_configs = match parse_socket_section(section) {
                    Ok(conf) => Some(conf),
                    Err(e) => return Err(format!("Error in file: {:?} :: {}", path, e)),
                };
            }
            "[Unit]" => {
                unit_config = Some(parse_unit_section(section, path));
            }
            "[Install]" => {
                install_config = Some(parse_install_section(section));
            }

            _ => panic!("Unknown section name: {}", name),
        }
    }

    // TODO handle install configs for sockets
    let _ = install_config;

    let (sock_name, services, sock_configs) = match socket_configs {
        Some(triple) => triple,
        None => return Err(format!("Didnt find socket config in file: {:?}", path)),
    };

    let conf = match unit_config {
        Some(conf) => conf,
        None => return Err(format!("Didn't find a unit config for file: {:?}", path)),
    };

    Ok(Unit {
        conf,
        id: chosen_id,
        install: Install::default(),
        specialized: UnitSpecialized::Socket(Socket {
            activated: false,
            name: sock_name,
            sockets: sock_configs,
            services,
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
    section: ParsedSection,
) -> Result<(String, Vec<String>, Vec<SocketConfig>), String> {
    let mut fdname: Option<String> = None;
    let mut socket_kinds: Vec<(u32, SocketKind)> = Vec::new();
    let mut services: Vec<String> = Vec::new();

    // TODO check that there is indeed exactly one value per name
    for (name, mut values) in section {
        match name.as_str() {
            "FILEDESCRIPTORNAME" => {
                fdname = Some(values.remove(0).1);
            }
            "LISTENSTREAM" => {
                for _ in 0..values.len() {
                    let (entry_num, value) = values.remove(0);
                    socket_kinds.push((entry_num, SocketKind::Stream(value)));
                }
            }
            "LISTENDATAGRAM" => {
                for _ in 0..values.len() {
                    let (entry_num, value) = values.remove(0);
                    socket_kinds.push((entry_num, SocketKind::Datagram(value)));
                }
            }
            "LISTENSEQUENTIALPACKET" => {
                for _ in 0..values.len() {
                    let (entry_num, value) = values.remove(0);
                    socket_kinds.push((entry_num, SocketKind::Sequential(value)));
                }
            }
            "LISTENFIFO" => {
                for _ in 0..values.len() {
                    let (entry_num, value) = values.remove(0);
                    socket_kinds.push((entry_num, SocketKind::Fifo(value)));
                }
            }
            "SERVICE" => {
                for _ in 0..values.len() {
                    let (_, value) = values.remove(0);
                    services.push(value);
                }
            }
            _ => panic!("Unknown parameter name: {}", name),
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
                    return Err(format!(
                        "No specialized config for fifo found for fifo addr: {}",
                        addr
                    ));
                }
            }
            SocketKind::Sequential(addr) => {
                if parse_unix_addr(addr).is_ok() {
                    SpecializedSocketConfig::UnixSocket(UnixSocketConfig::Sequential(addr.clone()))
                } else {
                    return Err(format!(
                        "No specialized config for socket found for socket addr: {}",
                        addr
                    ));
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
                    return Err(format!(
                        "No specialized config for socket found for socket addr: {}",
                        addr
                    ));
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
                    return Err(format!(
                        "No specialized config for socket found for socket addr: {}",
                        addr
                    ));
                }
            }
        };

        socket_configs.push(SocketConfig {
            kind,
            specialized,
            fd: None,
        });
    }

    let name = match fdname {
        Some(name) => name,
        None => "unknown".into(),
    };

    Ok((name, services, socket_configs))
}
