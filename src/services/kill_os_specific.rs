use crate::services::Service;

#[cfg(feature = "cgroups")]
use crate::platform::cgroup2;

pub fn kill(srvc: &mut Service, sig: nix::sys::signal::Signal) -> Result<(), String> {
    #[cfg(feature = "cgroups")]
    {
        if nix::unistd::getuid().is_root() {
            cgroup2::kill_cgroup(&srvc.platform_specific.cgroup_path, sig)
                .map_err(|e| format!("{}", e))?;
        }
    }
    let _ = srvc;
    let _ = sig;
    Ok(())
}
