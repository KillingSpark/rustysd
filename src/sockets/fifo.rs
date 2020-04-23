use std::{os::unix::io::AsRawFd, os::unix::io::FromRawFd, os::unix::io::RawFd};

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct FifoConfig {
    pub path: std::path::PathBuf,
}

impl FifoConfig {
    pub fn open(&self) -> Result<Box<dyn AsRawFd + Send + Sync>, String> {
        if self.path.exists() {
            std::fs::remove_file(&self.path)
                .map_err(|e| format!("Error removing file {:?}: {}", self.path, e))?;
        }
        let mode = nix::sys::stat::Mode::S_IRWXU;
        nix::unistd::mkfifo(&self.path, mode)
            .map_err(|e| format!("Error while creating fifo {:?}: {}", &self.path, e))?;

        // open NON_BLOCK so we dont wait for the other end of the fifo
        let mut open_flags = nix::fcntl::OFlag::empty();
        open_flags.insert(nix::fcntl::OFlag::O_RDWR);
        //open_flags.insert(nix::fcntl::OFlag::O_NONBLOCK);
        let fifo_fd = nix::fcntl::open(&self.path, open_flags, mode)
            .map_err(|e| format!("Error opening fifo file {:?}: {}", self.path, e))?;
        // need to make a file out of that so AsRawFd is implemented (it's not implmeneted for RawFd itself...)
        let fifo = unsafe { std::fs::File::from_raw_fd(fifo_fd) };
        Ok(Box::new(fifo))
    }

    pub fn close(&self, rawfd: RawFd) -> Result<(), String> {
        if self.path.exists() {
            std::fs::remove_file(&self.path)
                .map_err(|e| format!("Error removing file {:?}: {}", self.path, e))?;
        }
        super::close_raw_fd(rawfd);
        Ok(())
    }
}
