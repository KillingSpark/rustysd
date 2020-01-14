use crate::services::Service;

#[cfg(feature = "cgroups")]
use crate::platform::cgroups;

/// This is the place to do anything that is not standard unix but specific to one os. Like cgroups

pub fn pre_fork_os_specific(srvc: &mut Service) -> Result<(), String> {
    #[cfg(feature = "cgroups")]
    {
        if nix::unistd::getuid().is_root() {
            cgroups::get_or_make_freezer(
                &srvc.platform_specific.cgroupv1_freezer_path,
                &srvc.platform_specific.cgroupv2_unified_path,
                &srvc.platform_specific.relative_path,
            )
            .map_err(|e| format!("{}", e))?;
        }
    }
    let _ = srvc;
    Ok(())
}

pub fn post_fork_os_specific(srvc: &mut Service) -> Result<(), String> {
    #[cfg(feature = "cgroups")]
    {
        if nix::unistd::getuid().is_root() {
            let p = cgroups::get_or_make_freezer(
                &srvc.platform_specific.cgroupv1_freezer_path,
                &srvc.platform_specific.cgroupv2_unified_path,
                &srvc.platform_specific.relative_path,
            )
            .map_err(|e| format!("{}", e))?;
            cgroups::move_self_to_cgroup(&p).map_err(|e| format!("postfork os specific: {}", e))?;
        }
    }
    let _ = srvc;
    Ok(())
}
