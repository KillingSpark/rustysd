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

/// retrieve all controllers that are currently in this cgroup
pub fn get_available_controllers(
    cgroup_path: &std::path::PathBuf,
) -> Result<Vec<String>, CgroupError> {
    let cgroup_ctrls = cgroup_path.join("cgroup.controllers");
    let mut f = fs::File::open(&cgroup_ctrls).map_err(|e| CgroupError::IOErr(e))?;
    let mut buf = String::new();
    f.read_to_string(&mut buf)
        .map_err(|e| CgroupError::IOErr(e))?;

    Ok(buf.split('\n').map(|s| s.to_string()).collect())
}

/// enable controllers for child-cgroups
pub fn enable_controllers(
    cgroup_path: &std::path::PathBuf,
    controllers: &Vec<String>,
) -> Result<(), CgroupError> {
    let cgroup_subtreectl = cgroup_path.join("cgroup.subtree_control");
    let mut f = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&cgroup_subtreectl)
        .map_err(|e| CgroupError::IOErr(e))?;

    let mut buf = String::new();
    for ctl in controllers {
        buf.push_str(" +");
        buf.push_str(&ctl);
    }
    f.write_all(buf.as_bytes())
        .map_err(|e| CgroupError::IOErr(e))?;
    Ok(())
}

/// disable controllers for child-cgroups
pub fn disable_controllers(
    cgroup_path: &std::path::PathBuf,
    controllers: &Vec<String>,
) -> Result<(), CgroupError> {
    let cgroup_subtreectl = cgroup_path.join("cgroup.subtree_control");
    let mut f = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&cgroup_subtreectl)
        .map_err(|e| CgroupError::IOErr(e))?;

    let mut buf = String::new();
    for ctl in controllers {
        buf.push_str(" -");
        buf.push_str(&ctl);
    }
    f.write_all(buf.as_bytes())
        .map_err(|e| CgroupError::IOErr(e))?;
    Ok(())
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

fn write_freeze_state(
    cgroup_path: &std::path::PathBuf,
    desired_state: &str,
) -> Result<(), CgroupError> {
    let cgroup_freeze = cgroup_path.join("cgroup.freeze");
    if cgroup_freeze.exists() {
        return Err(CgroupError::IOErr(std::io::Error::from(
            std::io::ErrorKind::NotFound,
        )));
    }
    let mut f = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&cgroup_freeze)
        .map_err(|e| CgroupError::IOErr(e))?;

    f.write_all(desired_state.as_bytes())
        .map_err(|e| CgroupError::IOErr(e))?;
    Ok(())
}

pub fn freeze(cgroup_path: &std::path::PathBuf) -> Result<(), CgroupError> {
    let desired_state = "1";
    write_freeze_state(cgroup_path, desired_state)
}

pub fn thaw(cgroup_path: &std::path::PathBuf) -> Result<(), CgroupError> {
    let desired_state = "1";
    write_freeze_state(cgroup_path, desired_state)
}
