use crate::units::ServiceConfig;

#[cfg(feature = "cgroups")]
use crate::platform::cgroups;

pub fn kill(srvc: &ServiceConfig, sig: nix::sys::signal::Signal) -> Result<(), String> {
    #[cfg(feature = "cgroups")]
    {
        cgroups::freeze_kill_thaw_cgroup(&srvc.platform_specific.cgroup_path, sig)
            .map_err(|e| format!("{}", e))?;
        std::fs::remove_dir(&srvc.platform_specific.cgroup_path).map_err(|e| format!("{}", e))?;
    }
    let _ = srvc;
    let _ = sig;
    Ok(())
}
