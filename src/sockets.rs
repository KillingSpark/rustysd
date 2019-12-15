use std::{
    net::TcpListener,
    net::UdpSocket,
    os::unix::io::AsRawFd,
    os::unix::io::FromRawFd,
    os::unix::net::{UnixDatagram, UnixListener},
    sync::Arc,
};

use crate::units::*;

#[derive(Clone, Debug)]
pub enum SocketKind {
    Stream(String),
    Sequential(String),
    Datagram(String),
    Fifo(String),
}

#[derive(Clone, Debug)]
pub enum SpecializedSocketConfig {
    UnixSocket(UnixSocketConfig),
    Fifo(FifoConfig),
    TcpSocket(TcpSocketConfig),
    UdpSocket(UdpSocketConfig),
}

impl SpecializedSocketConfig {
    fn open(&self) -> Result<Arc<Box<dyn AsRawFd>>, String> {
        match self {
            SpecializedSocketConfig::UnixSocket(conf) => conf.open(),
            SpecializedSocketConfig::TcpSocket(conf) => conf.open(),
            SpecializedSocketConfig::UdpSocket(conf) => conf.open(),
            SpecializedSocketConfig::Fifo(conf) => conf.open(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum UnixSocketConfig {
    Stream(String),
    Sequential(String),
    Datagram(String),
}

#[derive(Clone, Debug)]
pub struct FifoConfig {
    pub path: std::path::PathBuf,
}

impl FifoConfig {
    fn open(&self) -> Result<Arc<Box<dyn AsRawFd>>, String> {
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
        Ok(Arc::new(Box::new(fifo)))
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
    fn open(&self) -> Result<Arc<Box<dyn AsRawFd>>, String> {
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
                Ok(Arc::new(Box::new(stream)))
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
                Ok(Arc::new(Box::new(stream)))
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

                //let addr_family = nix::sys::socket::AddressFamily::Unix;
                //let sock_type = nix::sys::socket::SockType::SeqPacket;
                //let flags = nix::sys::socket::SockFlag::empty(); //flags can be set by using the fnctl calls later if necessary
                let protocol = 0; // not really important, used to choose protocol but we dont support sockets where thats relevant

                let path = std::path::PathBuf::from(&path);
                let unix_addr = nix::sys::socket::UnixAddr::new(&path).unwrap();
                let sock_addr = nix::sys::socket::SockAddr::Unix(unix_addr);

                trace!("opening seqpacket unix socket: {:?}", path);
                // first create the socket
                // cant use nix::socket because they only allow tcp/udp as protocols
                // TODO make pull request and get a "Auto" = 0 member
                let fd = unsafe { libc::socket(libc::AF_UNIX, libc::SOCK_SEQPACKET, protocol) };
                // then bind the socket to the path
                nix::sys::socket::bind(fd, &sock_addr).unwrap();
                // then make the socket an accepting one
                nix::sys::socket::listen(fd, 128).unwrap();

                // return our own type until the std supports seuqntial packet unix sockets
                Ok(Arc::new(Box::new(UnixSeqPacket(Some(fd)))))
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct TcpSocketConfig {
    pub addr: std::net::SocketAddr,
}

impl TcpSocketConfig {
    fn open(&self) -> Result<Arc<Box<dyn AsRawFd>>, String> {
        trace!("opening tcp socket: {:?}", self.addr);
        let listener = TcpListener::bind(self.addr).unwrap();
        //need to stop the listener to drop which would close the filedescriptor
        Ok(Arc::new(Box::new(listener)))
    }
}

#[derive(Clone, Debug)]
pub struct UdpSocketConfig {
    pub addr: std::net::SocketAddr,
}

impl UdpSocketConfig {
    fn open(&self) -> Result<Arc<Box<dyn AsRawFd>>, String> {
        trace!("opening udp socket: {:?}", self.addr);
        let listener = UdpSocket::bind(self.addr).unwrap();
        //need to stop the listener to drop which would close the filedescriptor
        Ok(Arc::new(Box::new(listener)))
    }
}

#[derive(Debug)]
pub struct Socket {
    pub name: String,
    pub sockets: Vec<SocketConfig>,
    pub services: Vec<String>,
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
}

pub fn open_all_sockets(sockets: &mut SocketTable) -> std::io::Result<()> {
    for socket_unit in sockets.values_mut() {
        if let UnitSpecialized::Socket(socket) = &mut socket_unit.specialized {
            for idx in 0..socket.sockets.len() {
                let conf = &mut socket.sockets[idx];
                let as_raw_fd = conf.specialized.open().unwrap();
                // close these fd's on exec. They must not show up in child processes
                // the ńeeded fd's will be duped which unsets the flag again
                let new_fd = as_raw_fd.as_raw_fd();
                nix::fcntl::fcntl(
                    new_fd,
                    nix::fcntl::FcntlArg::F_SETFD(nix::fcntl::FdFlag::FD_CLOEXEC),
                )
                .unwrap();
                conf.fd = Some(as_raw_fd);
                //need to stop the listener to drop which would close the filedescriptor
            }
        }
    }

    Ok(())
}

pub fn apply_sockets_to_services(
    mut service_table: ServiceTable,
    socket_table: &SocketTable,
) -> Result<ServiceTable, String> {
    for sock_unit in socket_table.values() {
        let mut counter = 0;

        if let UnitSpecialized::Socket(sock) = &sock_unit.specialized {
            trace!("Searching services for socket: {}", sock_unit.conf.name());
            for srvc_unit in service_table.values_mut() {
                let srvc = &mut srvc_unit.specialized;
                if let UnitSpecialized::Service(srvc) = srvc {
                    // add sockets for services with the exact same name
                    if (srvc_unit.conf.name() == sock_unit.conf.name())
                        && !srvc.socket_names.contains(&sock.name)
                    {
                        trace!(
                            "add socket: {} to service: {}",
                            sock_unit.conf.name(),
                            srvc_unit.conf.name()
                        );

                        srvc.socket_names.push(sock.name.clone());
                        counter += 1;
                    }

                    // add sockets to services that specify that the socket belongs to them
                    if let Some(srvc_conf) = &srvc.service_config {
                        if srvc_conf.sockets.contains(&sock_unit.conf.name())
                            && !srvc.socket_names.contains(&sock.name)
                        {
                            trace!(
                                "add socket: {} to service: {}",
                                sock_unit.conf.name(),
                                srvc_unit.conf.name()
                            );
                            srvc.socket_names.push(sock.name.clone());
                            counter += 1;
                        }
                    }
                }
            }

            // add socket to the specified services
            for srvc_name in &sock.services {
                for srvc_unit in service_table.values_mut() {
                    let srvc = &mut srvc_unit.specialized;
                    if let UnitSpecialized::Service(srvc) = srvc {
                        if (*srvc_name == srvc_unit.conf.name())
                            && !srvc.socket_names.contains(&sock.name)
                        {
                            trace!(
                                "add socket: {} to service: {}",
                                sock_unit.conf.name(),
                                srvc_unit.conf.name()
                            );

                            srvc.socket_names.push(sock.name.clone());
                            counter += 1;
                        }
                    }
                }
            }
        }
        if counter > 1 {
            return Err(format!(
                "Added socket: {} to too many services (should be at most one): {}",
                sock_unit.conf.name(),
                counter
            ));
        }
        if counter == 0 {
            warn!("Added socket: {} to no service", sock_unit.conf.name());
        }
    }

    Ok(service_table)
}
