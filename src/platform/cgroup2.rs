use std::fs;
use std::io::Read;
use std::io::Write;

pub enum CgroupError {
    IOErr(std::io::Error),
    NixErr(nix::Error),
}

impl std::fmt::Display for CgroupError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let msg = match self {
            CgroupError::IOErr(e) => format!("io error: {}", e),
            CgroupError::NixErr(e) => format!("nix error: {}", e),
        };
        fmt.write_str(format!("{}", msg).as_str())
    }
}

/// creates the needed cgroup directories
pub fn make_new_cgroup_recursive(cgroup_path: &std::path::PathBuf) -> Result<(), CgroupError> {
    if !cgroup_path.exists() {
        fs::create_dir_all(cgroup_path).map_err(|e| CgroupError::IOErr(e))
    } else {
        Ok(())
    }
}

/// move a process into the cgroup. In rustysd the child process will call move_self for convenience
pub fn move_pid_to_cgroup(
    cgroup_path: &std::path::PathBuf,
    pid: nix::unistd::Pid,
) -> Result<(), CgroupError> {
    let cgroup_procs = cgroup_path.join("cgroup.procs");

    let mut f = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&cgroup_procs)
        .map_err(|e| CgroupError::IOErr(e))?;

    let pid_str = pid.as_raw().to_string();
    f.write(pid_str.as_bytes())
        .map_err(|e| CgroupError::IOErr(e))?;
    Ok(())
}

/// move this process into the cgroup. Used by rustysd after forking
pub fn move_self_to_cgroup(cgroup_path: &std::path::PathBuf) -> Result<(), CgroupError> {
    let pid = nix::unistd::getpid();
    move_pid_to_cgroup(cgroup_path, pid)
}

/// retrieve all pids that are currently in this cgroup
pub fn get_all_procs(
    cgroup_path: &std::path::PathBuf,
) -> Result<Vec<nix::unistd::Pid>, CgroupError> {
    let mut pids = Vec::new();
    let cgroup_procs = cgroup_path.join("cgroup.procs");
    let mut f = fs::File::open(&cgroup_procs).map_err(|e| CgroupError::IOErr(e))?;
    let mut buf = String::new();
    f.read_to_string(&mut buf)
        .map_err(|e| CgroupError::IOErr(e))?;

    for pid_str in buf.split('\n') {
        if pid_str.len() == 0 {
            break;
        }
        if let Ok(pid) = pid_str.parse::<i32>() {
            pids.push(nix::unistd::Pid::from_raw(pid));
        }
    }
    Ok(pids)
}

/// kill all processes that are currently in this cgroup
pub fn kill_cgroup(
    cgroup_path: &std::path::PathBuf,
    sig: nix::sys::signal::Signal,
) -> Result<(), CgroupError> {
    // TODO figure out how to freeze a cgroup so no new processes can be spawned while killing
    let pids = get_all_procs(cgroup_path)?;
    for pid in &pids {
        nix::sys::signal::kill(*pid, sig).map_err(|e| CgroupError::NixErr(e))?;
    }
    Ok(())
}
