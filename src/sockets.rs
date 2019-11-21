use std::os::unix::io::RawFd;
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

#[derive(Clone)]
pub struct UnixSocketConfig {
    pub path: std::path::PathBuf,
    pub listener: Option<Arc<UnixListener>>,
}

#[derive(Clone)]
pub struct Socket {
    pub sockets: Vec<(SocketConfig, Option<RawFd>)>,
}

pub fn open_all_sockets(
    sockets: &mut std::collections::HashMap<InternalId, Unit>,
) -> std::io::Result<()> {
    for (_, socket) in sockets {
        if let UnitSpecialized::Socket(socket) = &mut socket.specialized {
            for idx in 0..socket.sockets.len() {
                let (conf, fd) = &mut socket.sockets[idx];
                match &mut conf.specialized {
                    SpecializedSocketConfig::UnixSocket(unix_conf) => {
                        let spath = std::path::Path::new(&unix_conf.path);
                        // Delete old socket if necessary
                        if spath.exists() {
                            std::fs::remove_file(&spath).unwrap();
                        }

                        trace!("opening unix socket: {:?}", &unix_conf.path);
                        // Bind to socket
                        let stream = match UnixListener::bind(&spath) {
                            Err(_) => panic!("failed to bind socket"),
                            Ok(stream) => stream,
                        };
                        *fd = Some(stream.as_raw_fd());
                        //need to stop the listener to drop which would close the filedescriptor
                        unix_conf.listener = Some(Arc::new(stream));
                    }
                }
            }
        }
    }

    Ok(())
}
