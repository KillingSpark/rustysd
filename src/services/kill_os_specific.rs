use crate::services::Service;

#[cfg(feature = "cgroups")]
use crate::platform::cgroups;

pub fn kill(srvc: &mut Service, sig: nix::sys::signal::Signal) -> Result<(), String> {
    #[cfg(feature = "cgroups")]
    {
        if nix::unistd::getuid().is_root() {
            let p = cgroups::get_or_make_freezer(
                &srvc.platform_specific.cgroupv1_freezer_path,
                &srvc.platform_specific.cgroupv2_unified_path,
                &srvc.platform_specific.relative_path,
            )
            .map_err(|e| format!("{}", e))?;

            cgroups::freeze_kill_thaw_cgroup(&p, sig).map_err(|e| format!("{}", e))?;
        }
    }
    let _ = srvc;
    let _ = sig;
    Ok(())
}
