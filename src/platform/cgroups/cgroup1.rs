use log::trace;

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

fn write_freeze_state(
    cgroup_path: &std::path::PathBuf,
    desired_state: &str,
) -> Result<(), CgroupError> {
    let cgroup_freeze = cgroup_path.join("freezer.state");
    if !cgroup_freeze.exists() {
        return Err(CgroupError::IOErr(
            std::io::Error::from(std::io::ErrorKind::NotFound),
            format!("{:?}", cgroup_freeze),
        ));
    }

    trace!("Write {} to {:?}", desired_state, cgroup_freeze);
    let mut f = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&cgroup_freeze)
        .map_err(|e| CgroupError::IOErr(e, format!("{:?}", cgroup_freeze)))?;

    f.write_all(desired_state.as_bytes())
        .map_err(|e| CgroupError::IOErr(e, format!("{:?}", cgroup_freeze)))?;
    Ok(())
}

pub fn wait_frozen(cgroup_path: &std::path::PathBuf) -> Result<(), CgroupError> {
    let cgroup_freeze = cgroup_path.join("freezer.state");
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

        if buf.len() >= 6 {
            if buf[0..6] == *"FROZEN".as_bytes() {
                break;
            } else {
                trace!(
                    "Wait for frozen state. Read (): {}",
                    String::from_utf8(buf.clone()).unwrap()
                );
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    Ok(())
}

pub fn freeze(cgroup_path: &std::path::PathBuf) -> Result<(), CgroupError> {
    let desired_state = "FROZEN";
    write_freeze_state(cgroup_path, desired_state)
}

pub fn thaw(cgroup_path: &std::path::PathBuf) -> Result<(), CgroupError> {
    let desired_state = "THAWED";
    write_freeze_state(cgroup_path, desired_state)
}
