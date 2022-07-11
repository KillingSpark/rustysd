use std::{net::TcpListener, net::UdpSocket, os::unix::io::AsRawFd, os::unix::io::RawFd};

use log::trace;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct TcpSocketConfig {
    pub addr: std::net::SocketAddr,
}

impl TcpSocketConfig {
    pub fn open(&self) -> Result<Box<dyn AsRawFd + Send + Sync>, String> {
        trace!("opening tcp socket: {:?}", self.addr);
        let listener = TcpListener::bind(self.addr).unwrap();
        //need to stop the listener to drop which would close the filedescriptor
        Ok(Box::new(listener))
    }
    pub fn close(&self, rawfd: RawFd) -> Result<(), String> {
        super::close_raw_fd(rawfd);
        Ok(())
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct UdpSocketConfig {
    pub addr: std::net::SocketAddr,
}

impl UdpSocketConfig {
    pub fn open(&self) -> Result<Box<dyn AsRawFd + Send + Sync>, String> {
        trace!("opening udp socket: {:?}", self.addr);
        let listener = UdpSocket::bind(self.addr).unwrap();
        //need to stop the listener to drop which would close the filedescriptor
        Ok(Box::new(listener))
    }

    pub fn close(&self, rawfd: RawFd) -> Result<(), String> {
        super::close_raw_fd(rawfd);
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Socket {
    pub activated: bool,
}
