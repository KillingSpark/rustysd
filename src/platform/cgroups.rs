use std::fs;
use std::io::Read;

pub enum CgroupError {
    IOErr(std::io::Error),
    NixErr(nix::Error),
    NotMounted,
}

impl std::fmt::Display for CgroupError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let msg = match self {
            CgroupError::IOErr(e) => format!("io error: {}", e),
            CgroupError::NixErr(e) => format!("nix error: {}", e),
            CgroupError::NotMounted => "The freezer cgroup was not mounted".into(),
        };
        fmt.write_str(format!("{}", msg).as_str())
    }
}

fn use_v2(unified_path: &std::path::PathBuf) -> bool {
    unified_path.join("cgroup.freeze").exists()
}

/// creates the needed cgroup directories
pub fn get_or_make_freezer(
    freezer_path: &std::path::PathBuf,
    unified_path: &std::path::PathBuf,
    cgroup_path: &std::path::PathBuf,
) -> Result<std::path::PathBuf, CgroupError> {
    if use_v2(unified_path) {
        super::cgroup2::get_or_make_cgroup(unified_path, cgroup_path)
    } else {
        super::cgroup1::get_or_make_freezer(freezer_path, cgroup_path)
    }
}

/// move a process into the cgroup. In rustysd the child process will call move_self for convenience
pub fn move_pid_to_cgroup(
    cgroup_path: &std::path::PathBuf,
    pid: nix::unistd::Pid,
) -> Result<(), CgroupError> {
    if use_v2(cgroup_path) {
        super::cgroup2::move_pid_to_cgroup(cgroup_path, pid)
    } else {
        super::cgroup1::move_pid_to_cgroup(cgroup_path, pid)
    }
}

/// move this process into the cgroup. Used by rustysd after forking
pub fn move_self_to_cgroup(cgroup_path: &std::path::PathBuf) -> Result<(), CgroupError> {
    if use_v2(cgroup_path) {
        super::cgroup2::move_self_to_cgroup(cgroup_path)
    } else {
        super::cgroup1::move_self_to_cgroup(cgroup_path)
    }
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

/// kill all processes that are currently in this cgroup.
/// This makes sure that the cgroup is first completely frozen
/// so all processes will be killed and there is no chance of any
/// remaining
pub fn freeze_kill_thaw_cgroup(
    cgroup_path: &std::path::PathBuf,
    sig: nix::sys::signal::Signal,
) -> Result<(), CgroupError> {
    // TODO figure out how to freeze a cgroup so no new processes can be spawned while killing
    let use_v2 = use_v2(cgroup_path);
    if use_v2 {
        super::cgroup2::freeze(cgroup_path)
        super::cgroup2::wait_frozen(cgroup_path)
    } else {
        super::cgroup1::freeze(cgroup_path)
        super::cgroup1::wait_frozen(cgroup_path)
    }?;
    kill_cgroup(cgroup_path, sig)?;
    if use_v2 {
        super::cgroup2::thaw(cgroup_path)
    } else {
        super::cgroup1::thaw(cgroup_path)
    }
}

/// kill all processes that are currently in this cgroup.
/// You should use wait_frozen before or make in another way sure
/// there are no more processes spawned while killing
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

pub fn wait_frozen(cgroup_path: &std::path::PathBuf) -> Result<(), CgroupError> {
    if use_v2(cgroup_path) {
        super::cgroup2::wait_frozen(cgroup_path)
    } else {
        super::cgroup1::wait_frozen(cgroup_path)
    }
}

pub fn freeze(cgroup_path: &std::path::PathBuf) -> Result<(), CgroupError> {
    if use_v2(cgroup_path) {
        super::cgroup2::freeze(cgroup_path)
    } else {
        super::cgroup1::freeze(cgroup_path)
    }
}

pub fn thaw(cgroup_path: &std::path::PathBuf) -> Result<(), CgroupError> {
    if use_v2(cgroup_path) {
        super::cgroup2::thaw(cgroup_path)
    } else {
        super::cgroup1::thaw(cgroup_path)
    }
}
