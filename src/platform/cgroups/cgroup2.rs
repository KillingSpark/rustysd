use super::CgroupError;
use std::fs;
use std::io::Read;
use std::io::Write;

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
        .map_err(|e| CgroupError::IOErr(e, format!("{:?}", cgroup_procs)))?;

    let pid_str = pid.as_raw().to_string();
    f.write(pid_str.as_bytes())
        .map_err(|e| CgroupError::IOErr(e, format!("{:?}", cgroup_procs)))?;
    Ok(())
}

/// move this process into the cgroup. Used by rustysd after forking
pub fn move_self_to_cgroup(cgroup_path: &std::path::PathBuf) -> Result<(), CgroupError> {
    let pid = nix::unistd::getpid();
    move_pid_to_cgroup(cgroup_path, pid)
}

/// retrieve all controllers that are currently in this cgroup
#[allow(dead_code)]
pub fn get_available_controllers(
    cgroup_path: &std::path::PathBuf,
) -> Result<Vec<String>, CgroupError> {
    let cgroup_ctrls = cgroup_path.join("cgroup.controllers");
    let mut f = fs::File::open(&cgroup_ctrls)
        .map_err(|e| CgroupError::IOErr(e, format!("{:?}", cgroup_ctrls)))?;
    let mut buf = String::new();
    f.read_to_string(&mut buf)
        .map_err(|e| CgroupError::IOErr(e, format!("{:?}", cgroup_ctrls)))?;

    Ok(buf.split('\n').map(|s| s.to_string()).collect())
}

/// enable controllers for child-cgroups
#[allow(dead_code)]
pub fn enable_controllers(
    cgroup_path: &std::path::PathBuf,
    controllers: &Vec<String>,
) -> Result<(), CgroupError> {
    let cgroup_subtreectl = cgroup_path.join("cgroup.subtree_control");
    let mut f = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&cgroup_subtreectl)
        .map_err(|e| CgroupError::IOErr(e, format!("{:?}", cgroup_subtreectl)))?;

    let mut buf = String::new();
    for ctl in controllers {
        buf.push_str(" +");
        buf.push_str(&ctl);
    }
    f.write_all(buf.as_bytes())
        .map_err(|e| CgroupError::IOErr(e, format!("{:?}", cgroup_subtreectl)))?;
    Ok(())
}

/// disable controllers for child-cgroups
#[allow(dead_code)]
pub fn disable_controllers(
    cgroup_path: &std::path::PathBuf,
    controllers: &Vec<String>,
) -> Result<(), CgroupError> {
    let cgroup_subtreectl = cgroup_path.join("cgroup.subtree_control");
    let mut f = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&cgroup_subtreectl)
        .map_err(|e| CgroupError::IOErr(e, format!("{:?}", cgroup_subtreectl)))?;

    let mut buf = String::new();
    for ctl in controllers {
        buf.push_str(" -");
        buf.push_str(&ctl);
    }
    f.write_all(buf.as_bytes())
        .map_err(|e| CgroupError::IOErr(e, format!("{:?}", cgroup_subtreectl)))?;
    Ok(())
}

fn write_freeze_state(
    cgroup_path: &std::path::PathBuf,
    desired_state: &str,
) -> Result<(), CgroupError> {
    let cgroup_freeze = cgroup_path.join("cgroup.freeze");
    if !cgroup_freeze.exists() {
        return Err(CgroupError::IOErr(
            std::io::Error::from(std::io::ErrorKind::NotFound),
            format!("{:?}", cgroup_freeze),
        ));
    }

    let mut f = fs::OpenOptions::new()
        .read(false)
        .write(true)
        .open(&cgroup_freeze)
        .map_err(|e| CgroupError::IOErr(e, format!("{:?}", cgroup_freeze)))?;

    f.write_all(desired_state.as_bytes())
        .map_err(|e| CgroupError::IOErr(e, format!("{:?}", cgroup_freeze)))?;
    Ok(())
}

pub fn wait_frozen(cgroup_path: &std::path::PathBuf) -> Result<(), CgroupError> {
    let cgroup_freeze = cgroup_path.join("cgroup.freeze");
    let mut f = fs::OpenOptions::new()
        .read(true)
        .write(false)
        .open(&cgroup_freeze)
        .map_err(|e| CgroupError::IOErr(e, format!("{:?}", cgroup_freeze)))?;
    loop {
        freeze(cgroup_path)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)
            .map_err(|e| CgroupError::IOErr(e, format!("{:?}", cgroup_freeze)))?;
        if buf[0] == b'1' {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
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
