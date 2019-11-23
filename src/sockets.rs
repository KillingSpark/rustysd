use std::net::TcpListener;
use std::net::UdpSocket;
use std::os::unix::io::AsRawFd;
use std::os::unix::net::{UnixDatagram, UnixListener};
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
    TcpSocket(TcpSocketConfig),
    UdpSocket(UdpSocketConfig),
}

impl SpecializedSocketConfig {
    fn open(&self) -> Result<Arc<Box<AsRawFd>>, String> {
        match self {
            SpecializedSocketConfig::UnixSocket(conf) => conf.open(),
            SpecializedSocketConfig::TcpSocket(conf) => conf.open(),
            SpecializedSocketConfig::UdpSocket(conf) => conf.open(),
        }
    }
}

#[derive(Clone)]
pub struct UnixSocketConfig {
    pub kind: SocketKind,
}

impl UnixSocketConfig {
    fn open(&self) -> Result<Arc<Box<AsRawFd>>, String> {
        match &self.kind {
            SocketKind::Stream(path) => {
                let spath = std::path::Path::new(&path);
                // Delete old socket if necessary
                if spath.exists() {
                    std::fs::remove_file(&spath).unwrap();
                }

                trace!("opening streaming unix socket: {:?}", path);
                // Bind to socket
                let stream = match UnixListener::bind(&spath) {
                    Err(_) => panic!("failed to bind socket"),
                    Ok(stream) => stream,
                };
                //need to stop the listener to drop which would close the filedescriptor
                Ok(Arc::new(Box::new(stream)))
            }
            SocketKind::Datagram(path) => {
                let spath = std::path::Path::new(&path);
                // Delete old socket if necessary
                if spath.exists() {
                    std::fs::remove_file(&spath).unwrap();
                }

                trace!("opening datagram unix socket: {:?}", path);
                // Bind to socket
                let stream = match UnixDatagram::bind(&spath) {
                    Err(_) => panic!("failed to bind socket"),
                    Ok(stream) => stream,
                };
                //need to stop the listener to drop which would close the filedescriptor
                Ok(Arc::new(Box::new(stream)))
            }
            SocketKind::Sequential(_) => {
                unimplemented!("Sequential sockets are not implemented");
            }
        }
    }
}

#[derive(Clone)]
pub struct TcpSocketConfig {
    pub addr: std::net::SocketAddr,
}

impl TcpSocketConfig {
    fn open(&self) -> Result<Arc<Box<AsRawFd>>, String> {
        trace!("opening tcp socket: {:?}", self.addr);
        let listener = TcpListener::bind(self.addr).unwrap();
        //need to stop the listener to drop which would close the filedescriptor
        Ok(Arc::new(Box::new(listener)))
    }
}

#[derive(Clone)]
pub struct UdpSocketConfig {
    pub addr: std::net::SocketAddr,
}

impl UdpSocketConfig {
    fn open(&self) -> Result<Arc<Box<AsRawFd>>, String> {
        trace!("opening tcp socket: {:?}", self.addr);
        let listener = UdpSocket::bind(self.addr).unwrap();
        //need to stop the listener to drop which would close the filedescriptor
        Ok(Arc::new(Box::new(listener)))
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
