use std::os::unix::io::AsRawFd;
use std::os::unix::net::UnixListener;
use std::sync::Arc;

use crate::units::*;

#[derive(Clone)]
pub enum SocketKind {
    Stream(String),
    Sequential(String),
    Datagram(String),
}

#[derive(Clone)]
pub enum SpecializedSocketConfig {
    UnixSocket(UnixSocketConfig),
}

impl SpecializedSocketConfig {
    fn open(&self) -> Result<Arc<Box<AsRawFd>>, String> {
        match self {
            SpecializedSocketConfig::UnixSocket(conf) => conf.open(),
        }
    }
}

#[derive(Clone)]
pub struct UnixSocketConfig {
    pub path: std::path::PathBuf,
}

impl UnixSocketConfig {
    fn open(&self) -> Result<Arc<Box<AsRawFd>>, String> {
        let spath = std::path::Path::new(&self.path);
        // Delete old socket if necessary
        if spath.exists() {
            std::fs::remove_file(&spath).unwrap();
        }

        trace!("opening unix socket: {:?}", self.path);
        // Bind to socket
        let stream = match UnixListener::bind(&spath) {
            Err(_) => panic!("failed to bind socket"),
            Ok(stream) => stream,
        };
        //need to stop the listener to drop which would close the filedescriptor
        Ok(Arc::new(Box::new(stream)))
    }
}

#[derive(Clone)]
pub struct Socket {
    pub sockets: Vec<SocketConfig>,
}

pub fn open_all_sockets(
    sockets: &mut std::collections::HashMap<InternalId, Unit>,
) -> std::io::Result<()> {
    for (_, socket) in sockets {
        if let UnitSpecialized::Socket(socket) = &mut socket.specialized {
            for idx in 0..socket.sockets.len() {
                let conf = &mut socket.sockets[idx];
                let as_raw_fd = conf.specialized.open().unwrap();
                conf.fd = Some(as_raw_fd);
                //need to stop the listener to drop which would close the filedescriptor
            }
        }
    }

    Ok(())
}
