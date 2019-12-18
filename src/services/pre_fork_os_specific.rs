use crate::services::Service;

/// This is the place to do anything that is not standard unix but specific to one os. Like cgroups

pub fn pre_fork_os_specific(srvc: &mut Service, name: &str) -> Result<(), String> {
    #[cfg(not(feature = "cgroups"))]
    {
        setup_cgroup(srvc, name)?;
    }
    Ok(())
}

#[cfg(not(feature = "cgroups"))]
fn setup_cgroup(_srvc: &mut Service, _name: &str) -> Result<(), String> {
    // do cgroup stuff here
    Ok(())
}
