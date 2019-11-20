use std::os::unix::io::RawFd;

#[derive(Clone)]
pub struct SocketConfig {
    pub name: String,
    pub kind: SocketKind,
    pub specialized: SpecializedSocketConfig,
}

#[derive(Clone)]
pub enum SocketKind {
    Stream(String),
    Sequential(String),
    Datagram(String)
}

#[derive(Clone)]
pub enum SpecializedSocketConfig {
    UnixSocket(UnixSocketConfig)
}

#[derive(Clone)]
pub struct UnixSocketConfig {
    pub path: std::path::PathBuf,
}

pub struct Socket {
    pub id: crate::services::InternalId,
    pub filepath: std::path::PathBuf,
    pub unit_conf: Option<crate::services::UnitConfig>,
    pub sockets: Vec<(SocketConfig,Option<RawFd>)>,
}

impl Socket {
    pub fn name(&self) -> String {
        let name = self
            .filepath
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
        let name = name.trim_end_matches(".service").to_owned();

        name
    }
}