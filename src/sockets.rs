//! Socket related code. Opening of all different kinds, match sockets to services etc

use std::{
    net::TcpListener,
    net::UdpSocket,
    os::unix::io::AsRawFd,
    os::unix::io::FromRawFd,
    os::unix::io::RawFd,
    os::unix::net::{UnixDatagram, UnixListener},
};

use crate::fd_store::FDStore;
use crate::units::*;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum SocketKind {
    Stream(String),
    Sequential(String),
    Datagram(String),
    Fifo(String),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum SpecializedSocketConfig {
    UnixSocket(UnixSocketConfig),
    Fifo(FifoConfig),
    TcpSocket(TcpSocketConfig),
    UdpSocket(UdpSocketConfig),
}

impl SpecializedSocketConfig {
    fn open(&self) -> Result<Box<dyn AsRawFd + Send + Sync>, String> {
        match self {
            SpecializedSocketConfig::UnixSocket(conf) => conf.open(),
            SpecializedSocketConfig::TcpSocket(conf) => conf.open(),
            SpecializedSocketConfig::UdpSocket(conf) => conf.open(),
            SpecializedSocketConfig::Fifo(conf) => conf.open(),
        }
    }
    fn close(&self, rawfd: RawFd) -> Result<(), String> {
        match self {
            SpecializedSocketConfig::UnixSocket(conf) => conf.close(rawfd),
            SpecializedSocketConfig::TcpSocket(conf) => conf.close(rawfd),
            SpecializedSocketConfig::UdpSocket(conf) => conf.close(rawfd),
            SpecializedSocketConfig::Fifo(conf) => conf.close(rawfd),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum UnixSocketConfig {
    Stream(String),
    Sequential(String),
    Datagram(String),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct FifoConfig {
    pub path: std::path::PathBuf,
}

impl FifoConfig {
    fn open(&self) -> Result<Box<dyn AsRawFd + Send + Sync>, String> {
        if self.path.exists() {
            std::fs::remove_file(&self.path).unwrap();
        }
        let mode = nix::sys::stat::Mode::S_IRWXU;
        nix::unistd::mkfifo(&self.path, mode)
            .map_err(|e| format!("Error while creating fifo {:?}: {}", &self.path, e))?;

        // open NON_BLOCK so we dont wait for the other end of the fifo
        let mut open_flags = nix::fcntl::OFlag::empty();
        open_flags.insert(nix::fcntl::OFlag::O_RDWR);
        //open_flags.insert(nix::fcntl::OFlag::O_NONBLOCK);
        let fifo_fd = nix::fcntl::open(&self.path, open_flags, mode).unwrap();
        // need to make a file out of that so AsRawFd is implemented (it's not implmeneted for RawFd itself...)
        let fifo = unsafe { std::fs::File::from_raw_fd(fifo_fd) };
        Ok(Box::new(fifo))
    }

    fn close(&self, rawfd: RawFd) -> Result<(), String> {
        std::fs::remove_file(&self.path)
            .map_err(|e| format!("Error removing file {:?}: {}", self.path, e))?;
        nix::unistd::close(rawfd).unwrap();
        Ok(())
    }
}

#[derive(Debug)]
struct UnixSeqPacket(Option<i32>);

impl AsRawFd for UnixSeqPacket {
    fn as_raw_fd(&self) -> i32 {
        self.0.unwrap()
    }
}

impl Drop for UnixSeqPacket {
    fn drop(&mut self) {
        self.close();
    }
}

impl UnixSeqPacket {
    fn close(&mut self) {
        if let Some(fd) = self.0 {
            if let Err(e) = nix::unistd::close(fd) {
                error!("Error while closing unix sequential packet socket: {}", e);
            }
        }
        self.0 = None;
    }
}

impl UnixSocketConfig {
    fn close(&self, rawfd: RawFd) -> Result<(), String> {
        let strpath = match self {
            UnixSocketConfig::Stream(s) => s,
            UnixSocketConfig::Datagram(s) => s,
            UnixSocketConfig::Sequential(s) => s,
        };
        let path = std::path::PathBuf::from(strpath);
        std::fs::remove_file(&path)
            .map_err(|e| format!("Error removing file {:?}: {}", path, e))?;
        nix::unistd::close(rawfd)
            .map_err(|e| format!("Error closing raw fd for socket {}: {}", strpath, e))?;
        Ok(())
    }

    fn open(&self) -> Result<Box<dyn AsRawFd + Send + Sync>, String> {
        match self {
            UnixSocketConfig::Stream(path) => {
                let spath = std::path::Path::new(&path);
                // Delete old socket if necessary
                if spath.exists() {
                    std::fs::remove_file(&spath).unwrap();
                }

                if let Some(parent) = spath.parent() {
                    if !parent.exists() {
                        std::fs::create_dir_all(parent).map_err(|e| {
                            format!("Error creating UnixSocket directory {:?} : {}", parent, e)
                        })?;
                    }
                }

                trace!("opening streaming unix socket: {:?}", path);
                // Bind to socket
                let stream = match UnixListener::bind(&spath) {
                    Err(e) => panic!(format!("failed to bind socket: {}", e)),
                    Ok(stream) => stream,
                };
                //need to stop the listener to drop which would close the filedescriptor
                Ok(Box::new(stream))
            }
            UnixSocketConfig::Datagram(path) => {
                let spath = std::path::Path::new(&path);
                // Delete old socket if necessary
                if spath.exists() {
                    std::fs::remove_file(&spath).unwrap();
                }

                if let Some(parent) = spath.parent() {
                    if !parent.exists() {
                        std::fs::create_dir_all(parent).map_err(|e| {
                            format!("Error creating UnixSocket directory {:?} : {}", parent, e)
                        })?;
                    }
                }

                trace!("opening datagram unix socket: {:?}", path);
                // Bind to socket
                let stream = match UnixDatagram::bind(&spath) {
                    Err(e) => panic!(format!("failed to bind socket: {}", e)),
                    Ok(stream) => stream,
                };
                //need to stop the listener to drop which would close the filedescriptor
                Ok(Box::new(stream))
            }
            UnixSocketConfig::Sequential(path) => {
                let spath = std::path::Path::new(&path);
                // Delete old socket if necessary
                if spath.exists() {
                    std::fs::remove_file(&spath).unwrap();
                }

                if let Some(parent) = spath.parent() {
                    if !parent.exists() {
                        std::fs::create_dir_all(parent).map_err(|e| {
                            format!("Error creating UnixSocket directory {:?} : {}", parent, e)
                        })?;
                    }
                }

                let path = std::path::PathBuf::from(&path);
                trace!("opening datagram unix socket: {:?}", path);
                match crate::platform::make_seqpacket_socket(&path) {
                    Ok(fd) => {
                        // return our own type until the std supports sequential packet unix sockets
                        Ok(Box::new(UnixSeqPacket(Some(fd))))
                    }
                    Err(e) => Err(e),
                }
            }
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct TcpSocketConfig {
    pub addr: std::net::SocketAddr,
}

impl TcpSocketConfig {
    fn open(&self) -> Result<Box<dyn AsRawFd + Send + Sync>, String> {
        trace!("opening tcp socket: {:?}", self.addr);
        let listener = TcpListener::bind(self.addr).unwrap();
        //need to stop the listener to drop which would close the filedescriptor
        Ok(Box::new(listener))
    }
    fn close(&self, rawfd: RawFd) -> Result<(), String> {
        nix::unistd::close(rawfd)
            .map_err(|e| format!("Error closing raw fd for socket {}: {}", self.addr, e))
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct UdpSocketConfig {
    pub addr: std::net::SocketAddr,
}

impl UdpSocketConfig {
    fn open(&self) -> Result<Box<dyn AsRawFd + Send + Sync>, String> {
        trace!("opening udp socket: {:?}", self.addr);
        let listener = UdpSocket::bind(self.addr).unwrap();
        //need to stop the listener to drop which would close the filedescriptor
        Ok(Box::new(listener))
    }

    fn close(&self, rawfd: RawFd) -> Result<(), String> {
        nix::unistd::close(rawfd)
            .map_err(|e| format!("Error closing raw fd for socket {}: {}", self.addr, e))
    }
}

#[derive(Clone, Debug)]
pub struct Socket {
    pub name: String,
    pub sockets: Vec<SocketConfig>,
    pub services: Vec<String>,
    pub activated: bool,
}

impl Socket {
    pub fn build_name_list(&self) -> String {
        let mut name_list = String::with_capacity(
            self.name.as_bytes().len() * self.sockets.len() + self.sockets.len(),
        );
        name_list.push_str(&self.name);
        for _ in 0..self.sockets.len() - 1 {
            name_list.push(':');
            name_list.push_str(&self.name);
        }
        name_list
    }

    pub fn open_all(
        &mut self,
        name: String,
        id: UnitId,
        fd_store: &mut FDStore,
    ) -> std::io::Result<()> {
        let mut fds = Vec::new();
        for idx in 0..self.sockets.len() {
            let conf = &mut self.sockets[idx];
            let as_raw_fd = conf.specialized.open().unwrap();
            // close these fd's on exec. They must not show up in child processes
            // the ńeeded fd's will be duped which unsets the flag again
            let new_fd = as_raw_fd.as_raw_fd();
            nix::fcntl::fcntl(
                new_fd,
                nix::fcntl::FcntlArg::F_SETFD(nix::fcntl::FdFlag::FD_CLOEXEC),
            )
            .unwrap();
            fds.push((id, self.name.clone(), as_raw_fd));
            //need to stop the listener to drop which would close the filedescriptor
        }
        trace!(
            "Opened all sockets: {:?}",
            fds.iter()
                .map(|(_, _, fd)| fd.as_raw_fd())
                .collect::<Vec<_>>(),
        );
        fd_store.insert_global(name, fds);
        Ok(())
    }

    pub fn close_all(&mut self, name: String, fd_store: &mut FDStore) -> Result<(), String> {
        if let Some(fds) = fd_store.remove_global(&name) {
            for idx in 0..fds.len() {
                self.sockets[idx]
                    .specialized
                    .close(fds[idx].2.as_raw_fd())?;
            }
        }
        Ok(())
    }
}
