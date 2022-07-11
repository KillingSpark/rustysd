//! This module provides methods to manage processes with cgroups. Not resource management but reliable tracking of services.
//! It dynamically decides wether cgroups v1 or v2 should be used.
//!
//! The cgroup paths created by get_own_freezer return a path that is inside the cgroup that contains rustysd itself. With the naming scheme of the freezer
//! cgroups we should mostly comply to the guidelines here https://www.freedesktop.org/wiki/Software/systemd/PaxControlGroups/

use std::fs;
use std::io::Read;

use log::trace;

mod cgroup1;
mod cgroup2;

#[derive(Debug)]
pub enum CgroupError {
    IOErr(std::io::Error, String),
    NixErr(nix::Error),
    NotMounted,
}

impl std::fmt::Display for CgroupError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        let msg = match self {
            CgroupError::IOErr(e, f) => format!("io error: {}, file: {}", e, f),
            CgroupError::NixErr(e) => format!("nix error: {}", e),
            CgroupError::NotMounted => "The freezer cgroup was not mounted".into(),
        };
        fmt.write_str(format!("{}", msg).as_str())
    }
}

fn use_v2(cgroup_path: &std::path::PathBuf) -> bool {
    let freeze_file = cgroup_path.join("cgroup.freeze");
    let exists = freeze_file.exists();
    trace!("{:?} exists: {}", freeze_file, exists);
    exists
}

const OWN_CGROUP_NAME: &str = "rustysd_self";

/// moves rustysd into own cgroup if v2 is used
///
/// This is necessary because cgroupv2 discourages processes in cgroups that are not leafes
pub fn move_to_own_cgroup(base_path: &std::path::PathBuf) -> Result<(), CgroupError> {
    trace!("Move rustysd to own manager cgroup");
    let proc_content = std::fs::read_to_string("/proc/self/cgroup").unwrap();
    let proc_content_lines = proc_content.split('\n').collect::<Vec<_>>();
    let v2path = get_own_cgroup_v2(&proc_content_lines);
    trace!("V2 path: {:?}", v2path);
    if let Some(v2path) = v2path {
        let base_path = base_path.join("unified");
        let absolute_v2path = base_path.join(v2path);
        let rustysd_subgroup = absolute_v2path.join(format!("rustysd_{}", nix::unistd::getpid()));
        let manager_cgroup = rustysd_subgroup.join(OWN_CGROUP_NAME);
        trace!("Manager path: {:?}", manager_cgroup);
        if !manager_cgroup.exists() {
            std::fs::create_dir_all(&manager_cgroup)
                .map_err(|e| CgroupError::IOErr(e, format!("{:?}", manager_cgroup)))?;
        }
        move_self_to_cgroup(&manager_cgroup)?;
    }
    Ok(())
}

pub fn move_out_of_own_cgroup(base_path: &std::path::PathBuf) -> Result<(), CgroupError> {
    let proc_content = std::fs::read_to_string("/proc/self/cgroup").unwrap();
    let proc_content_lines = proc_content.split('\n').collect::<Vec<_>>();
    if let Some(v2path) = crate::platform::cgroups::get_own_cgroup_v2(&proc_content_lines) {
        let absolute_v2path = base_path.join(v2path);
        let mut parent_group = absolute_v2path.clone();
        parent_group.pop();
        trace!("Move rustysd to parent cgroup: {:?}", parent_group);
        crate::platform::cgroups::move_self_to_cgroup(&parent_group)?;

        let self_cgroup = absolute_v2path.join("rustysd_self");
        trace!("Remove manager cgroup: {:?}", self_cgroup);
        std::fs::remove_dir(&self_cgroup)
            .map_err(|e| CgroupError::IOErr(e, format!("{:?}", self_cgroup)))?;

        trace!("Remove rustysd managed cgroup: {:?}", absolute_v2path);
        std::fs::remove_dir(&absolute_v2path)
            .map_err(|e| CgroupError::IOErr(e, format!("{:?}", absolute_v2path)))?;
    }
    Ok(())
}

/// base_path should normally be /sys/fs/cgroup
///
/// Tries to get the most sensible path to create our own cgroup under.
/// Depending on whether cgroupv2 freezing is available It's either a path in
/// 1. /sys/fs/cgroup/freezer
/// 1. /sys/fs/cgroup/unified
///
/// The concrete path will be some sub-directory depending on the cgroup rustysd has been started in
pub fn get_own_freezer(base_path: &std::path::PathBuf) -> Result<std::path::PathBuf, CgroupError> {
    let proc_content = std::fs::read_to_string("/proc/self/cgroup").unwrap();
    let proc_content_lines = proc_content.split('\n').collect::<Vec<_>>();

    let v1path = get_own_cgroup_v1(&proc_content_lines);
    let v1_full_path = base_path.join("freezer").join(v1path);
    trace!("v1 cgroup: {:?}", v1_full_path);

    let v2path = get_own_cgroup_v2(&proc_content_lines);

    // prefer v2 path but fall back to v1 freezer
    let cgroup_path = if let Some(v2path) = v2path {
        let v2_full_path = base_path.join("unified").join(v2path);
        trace!("v2 cgroup: {:?}", v2_full_path);

        // If v2 group exists but we cant freeze it we still need to use the v1 controller
        if v2_full_path.join("cgroup.freeze").exists() {
            v2_full_path
        } else {
            v1_full_path
        }
    } else {
        v1_full_path
    };

    trace!("Own cgroup: {:?}", cgroup_path);

    fs::create_dir_all(&cgroup_path)
        .map_err(|e| CgroupError::IOErr(e, format!("{:?}", cgroup_path)))?;

    Ok(cgroup_path)
}

/// cgroup v2 appears in /proc/self/cgroup as 0::/path/to/cgroup
/// but the path is relative to the mount point of cgroups (/sys/fs/cgroup/unified).
pub fn get_own_cgroup_v2(proc_cgroup_content: &[&str]) -> Option<std::path::PathBuf> {
    for line in proc_cgroup_content {
        if line.starts_with("0::") {
            let path = &line[3..];
            // if we are already in the manager cgroup ignore that one. Return the managed cgroup
            let path = path.trim_end_matches(OWN_CGROUP_NAME);
            // ignore leading "/"
            let path = std::path::PathBuf::from(&path[1..]);
            return Some(path);
        }
    }
    None
}

/// Try to find the cgroup path for the freezer controller
/// If we are in / for freezer find the longest path used in any other cgroup and use that.
///
/// cgroups v1 by convention use the same (or a subset) directory trees under each controller so using the
/// longest path gives us the most specialized categorization and is probably what others would expect rustysd to do?
fn get_own_cgroup_v1(proc_cgroup_content: &[&str]) -> std::path::PathBuf {
    let mut freezer_path = None;
    let mut longest_path = "/".to_owned();

    for line in proc_cgroup_content {
        let triple = line.split(':').collect::<Vec<_>>();
        if triple.len() == 3 {
            let _id = triple[0];
            let controller = triple[1];
            let path = triple[2];

            if controller.eq("freezer") {
                // ignore leading "/"
                let path = &path[1..];
                freezer_path = Some(std::path::PathBuf::from(path));
            }

            if path.len() > longest_path.len() {
                longest_path = path.to_owned();
            }
        }
    }

    if let Some(p) = freezer_path {
        p
    } else {
        // ignore leading "/"
        std::path::PathBuf::from(&longest_path[1..])
    }
}

/// move a process into the cgroup. In rustysd the child process will call move_self for convenience
pub fn move_pid_to_cgroup(
    cgroup_path: &std::path::PathBuf,
    pid: nix::unistd::Pid,
) -> Result<(), CgroupError> {
    if use_v2(cgroup_path) {
        cgroup2::move_pid_to_cgroup(cgroup_path, pid)
    } else {
        cgroup1::move_pid_to_cgroup(cgroup_path, pid)
    }
}

/// move this process into the cgroup. Used by rustysd after forking
pub fn move_self_to_cgroup(cgroup_path: &std::path::PathBuf) -> Result<(), CgroupError> {
    if use_v2(cgroup_path) {
        cgroup2::move_self_to_cgroup(cgroup_path)
    } else {
        cgroup1::move_self_to_cgroup(cgroup_path)
    }
}

/// retrieve all pids that are currently in this cgroup
pub fn get_all_procs(
    cgroup_path: &std::path::PathBuf,
) -> Result<Vec<nix::unistd::Pid>, CgroupError> {
    let mut pids = Vec::new();
    let cgroup_procs = cgroup_path.join("cgroup.procs");
    let mut f = fs::File::open(&cgroup_procs)
        .map_err(|e| CgroupError::IOErr(e, format!("{:?}", cgroup_procs)))?;
    let mut buf = String::new();
    f.read_to_string(&mut buf)
        .map_err(|e| CgroupError::IOErr(e, format!("{:?}", cgroup_procs)))?;

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
    trace!("Freeze cgroup: {:?}", cgroup_path);
    if use_v2 {
        cgroup2::freeze(cgroup_path)?;
        cgroup2::wait_frozen(cgroup_path)?;
    } else {
        cgroup1::freeze(cgroup_path)?;
        cgroup1::wait_frozen(cgroup_path)?;
    }
    trace!("Kill cgroup: {:?}", cgroup_path);
    kill_cgroup(cgroup_path, sig)?;
    trace!("Thaw cgroup: {:?}", cgroup_path);
    if use_v2 {
        cgroup2::thaw(cgroup_path)
    } else {
        cgroup1::thaw(cgroup_path)
    }
}

pub fn remove_cgroup(cgroup_path: &std::path::PathBuf) -> Result<(), CgroupError> {
    fs::remove_dir(&cgroup_path).map_err(|e| CgroupError::IOErr(e, format!("{:?}", cgroup_path)))
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
        cgroup2::wait_frozen(cgroup_path)
    } else {
        cgroup1::wait_frozen(cgroup_path)
    }
}

pub fn freeze(cgroup_path: &std::path::PathBuf) -> Result<(), CgroupError> {
    if use_v2(cgroup_path) {
        cgroup2::freeze(cgroup_path)
    } else {
        cgroup1::freeze(cgroup_path)
    }
}

pub fn thaw(cgroup_path: &std::path::PathBuf) -> Result<(), CgroupError> {
    if use_v2(cgroup_path) {
        cgroup2::thaw(cgroup_path)
    } else {
        cgroup1::thaw(cgroup_path)
    }
}
