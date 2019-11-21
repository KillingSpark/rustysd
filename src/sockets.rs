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
    Datagram(String),
}

#[derive(Clone)]
pub enum SpecializedSocketConfig {
    UnixSocket(UnixSocketConfig),
}

use std::os::unix::net::UnixListener;
use std::rc::Rc;
#[derive(Clone)]
pub struct UnixSocketConfig {
    pub path: std::path::PathBuf,
    pub listener: Option<Rc<UnixListener>>,
}

pub struct Socket {
    pub filepath: std::path::PathBuf,
    pub sockets: Vec<(SocketConfig, Option<RawFd>)>,
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
        let name = name.trim_end_matches(".socket").to_owned();

        name
    }
}

use std::os::unix::io::AsRawFd;
pub fn open_all_sockets(
    sockets: &mut std::collections::HashMap<crate::services::InternalId, crate::unit_parser::Unit>,
) -> std::io::Result<()> {
    for (_, socket) in sockets {
        if let crate::unit_parser::UnitSpecialized::Socket(socket) = &mut socket.specialized {
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
                        unix_conf.listener = Some(Rc::new(stream));
                    }
                }
            }
        }
    }

    Ok(())
}
