use crate::services::Service;

#[cfg(feature = "cgroups")]
use crate::platform::cgroup2;

/// This is the place to do anything that is not standard unix but specific to one os. Like cgroups

pub fn pre_fork_os_specific(srvc: &mut Service) -> Result<(), String> {
    #[cfg(feature = "cgroups")]
    {
        if nix::unistd::getuid().is_root() {
            let cgroupv2_path = srvc.platform_specific.cgroupv2_path.join(&srvc.platform_specific.relative_path);
            cgroup2::make_new_cgroup_recursive(&cgroupv2_path)
            .map_err(|e| format!("prefork os specific: {}", e))?;
        }
    }
    let _ = srvc;
    Ok(())
}

pub fn post_fork_os_specific(srvc: &mut Service) -> Result<(), String> {
    #[cfg(feature = "cgroups")]
    {
        if nix::unistd::getuid().is_root() {
            let cgroupv2_path = srvc.platform_specific.cgroupv2_path.join(&srvc.platform_specific.relative_path);
            cgroup2::move_self_to_cgroup(&cgroupv2_path)
                .map_err(|e| format!("postfork os specific: {}", e))?;
        }
    }
    let _ = srvc;
    Ok(())
}
