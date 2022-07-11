//! Socket related code. Opening of all different kinds, match sockets to services etc

mod fifo;
mod network_sockets;
mod unix_sockets;
pub use fifo::*;
use log::trace;
pub use network_sockets::*;
pub use unix_sockets::*;

use std::{os::unix::io::AsRawFd, os::unix::io::RawFd};

use crate::fd_store::FDStore;
use crate::units::*;

pub fn close_raw_fd(fd: RawFd) {
    loop {
        match nix::unistd::close(fd) {
            Ok(()) => break,
            Err(e) => {
                if let nix::errno::Errno::EBADF = e {
                    break;
                }
                // Other errors (EINTR and EIO) mean that we should try again
            }
        }
    }
}

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

impl Socket {
    pub fn build_name_list(&self, conf: SocketConfig) -> String {
        let mut name_list = String::with_capacity(
            conf.filedesc_name.as_bytes().len() * conf.sockets.len() + conf.sockets.len(),
        );
        name_list.push_str(&conf.filedesc_name);
        for _ in 0..conf.sockets.len() - 1 {
            name_list.push(':');
            name_list.push_str(&conf.filedesc_name);
        }
        name_list
    }

    pub fn open_all(
        &mut self,
        conf: &SocketConfig,
        name: String,
        id: UnitId,
        fd_store: &mut FDStore,
    ) -> std::io::Result<()> {
        let mut fds = Vec::new();
        for idx in 0..conf.sockets.len() {
            let single_conf = &conf.sockets[idx];
            let as_raw_fd = single_conf.specialized.open().unwrap();
            // close these fd's on exec. They must not show up in child processes
            // the ńeeded fd's will be duped which unsets the flag again
            let new_fd = as_raw_fd.as_raw_fd();
            nix::fcntl::fcntl(
                new_fd,
                nix::fcntl::FcntlArg::F_SETFD(nix::fcntl::FdFlag::FD_CLOEXEC),
            )
            .unwrap();
            fds.push((id.clone(), conf.filedesc_name.clone(), as_raw_fd));
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

    pub fn close_all(
        &mut self,
        conf: &SocketConfig,
        name: String,
        fd_store: &mut FDStore,
    ) -> Result<(), String> {
        if let Some(fds) = fd_store.remove_global(&name) {
            for idx in 0..fds.len() {
                conf.sockets[idx]
                    .specialized
                    .close(fds[idx].2.as_raw_fd())?;
            }
        }
        Ok(())
    }
}
