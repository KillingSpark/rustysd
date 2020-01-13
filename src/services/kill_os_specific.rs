use crate::services::Service;

#[cfg(feature = "cgroups")]
use crate::platform::cgroup2;

pub fn kill(srvc: &mut Service, sig: nix::sys::signal::Signal) -> Result<(), String> {
    #[cfg(feature = "cgroups")]
    {
        if nix::unistd::getuid().is_root() {
            let cgroupv2_path = srvc.platform_specific.cgroupv2_path.join(&srvc.platform_specific.relative_path);
            cgroup2::kill_cgroup(&cgroupv2_path, sig)
                .map_err(|e| format!("{}", e))?;
        }
    }
    let _ = srvc;
    let _ = sig;
    Ok(())
}
