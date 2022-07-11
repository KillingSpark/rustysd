use std::{
    os::unix::io::AsRawFd,
    os::unix::io::RawFd,
    os::unix::net::{UnixDatagram, UnixListener},
};

use log::trace;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum UnixSocketConfig {
    Stream(String),
    Sequential(String),
    Datagram(String),
}

#[derive(Debug)]
struct UnixSeqPacket(Option<i32>, std::path::PathBuf);

impl AsRawFd for UnixSeqPacket {
    fn as_raw_fd(&self) -> i32 {
        self.0.unwrap()
    }
}

impl Drop for UnixSeqPacket {
    fn drop(&mut self) {
        if self.1.exists() {
            self.close();
            std::fs::remove_file(&self.1).unwrap();
        }
    }
}

impl UnixSeqPacket {
    fn close(&mut self) {
        if let Some(fd) = self.0 {
            super::close_raw_fd(fd);
        }
        self.0 = None;
    }
}

impl UnixSocketConfig {
    pub fn close(&self, rawfd: RawFd) -> Result<(), String> {
        let strpath = match self {
            UnixSocketConfig::Stream(s) => s,
            UnixSocketConfig::Datagram(s) => s,
            UnixSocketConfig::Sequential(s) => s,
        };
        let path = std::path::PathBuf::from(strpath);
        if path.exists() {
            std::fs::remove_file(&path)
                .map_err(|e| format!("Error removing file {:?}: {}", path, e))?;
        }

        super::close_raw_fd(rawfd);
        Ok(())
    }

    pub fn open(&self) -> Result<Box<dyn AsRawFd + Send + Sync>, String> {
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
                    Err(e) => panic!("failed to bind socket: {}", e),
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
                    Err(e) => panic!("failed to bind socket: {}", e),
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
                        Ok(Box::new(UnixSeqPacket(Some(fd), path.clone())))
                    }
                    Err(e) => Err(e),
                }
            }
        }
    }
}
