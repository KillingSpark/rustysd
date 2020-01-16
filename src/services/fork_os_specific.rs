use crate::services::Service;

#[cfg(feature = "cgroups")]
use crate::platform::cgroups;

/// This is the place to do anything that is not standard unix but specific to one os. Like cgroups

pub fn pre_fork_os_specific(srvc: &mut Service) -> Result<(), String> {
    #[cfg(feature = "cgroups")]
    {
        if nix::unistd::getuid().is_root() {
            std::fs::create_dir_all(&srvc.platform_specific.cgroup_path).map_err(|e| {
                format!(
                    "Couldnt create service cgroup ({:?}): {}",
                    srvc.platform_specific.cgroup_path, e
                )
            })?;
        }
    }
    let _ = srvc;
    Ok(())
}

pub fn post_fork_os_specific(srvc: &mut Service) -> Result<(), String> {
    #[cfg(feature = "cgroups")]
    {
        if nix::unistd::getuid().is_root() {
            cgroups::move_self_to_cgroup(&srvc.platform_specific.cgroup_path)
                .map_err(|e| format!("postfork os specific: {}", e))?;
        }
    }
    let _ = srvc;
    Ok(())
}
