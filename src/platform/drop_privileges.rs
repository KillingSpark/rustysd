use nix::unistd::setresgid;
use nix::unistd::setresuid;
use nix::unistd::Gid;
use nix::unistd::Uid;
use std::io::Read;

/// This sequence should drop all privileges the root process might have had. I think this is how systemd does it too.
/// They additionally have some checking if setgroups is possible
///
/// I dont think this needs to explicitly drop any capabilities on linux. At least thats how I understood the man page
pub fn drop_privileges(gid: Gid, supp_gids: &Vec<Gid>, uid: Uid) -> Result<(), String> {
    setresgid(gid, gid, gid).map_err(|e| format!("Error while setting groupid: {}", e))?;
    maybe_set_groups(supp_gids)?;
    setresuid(uid, uid, uid).map_err(|e| format!("Error while setting userid: {}", e))?;
    Ok(())
}

const ALLOW_READ: [u8; 5] = [b'a', b'l', b'l', b'o', b'w'];

fn maybe_set_groups(supp_gids: &Vec<Gid>) -> Result<(), String> {
    if can_drop_groups()? {
        nix::unistd::setgroups(supp_gids)
            .map_err(|e| format!("Error while calling setgroups: {}", e))
    } else {
        // TODO check if this is sensible.
        // We just ignore groups if the kernel says we cant drop them. Maybe we should just not start the servcie then?
        // systemd seems to do it like this here
        // https://github.com/systemd/systemd/blob/master/src/basic/user-util.c
        Ok(())
    }
}

fn can_drop_groups() -> Result<bool, String> {
    let kernel_iface_path = std::path::PathBuf::from("/proc/self/setgroups");

    if !kernel_iface_path.exists() {
        // assume true since we cant check
        Ok(true)
    } else {
        let mut buf = [0u8; 5];
        let mut file = std::fs::File::open(&kernel_iface_path).map_err(|e| {
            format!(
                "Error while opening file: {:?} to check if we can call setgroups: {}",
                kernel_iface_path, e
            )
        })?;
        file.read(&mut buf[..]).unwrap();
        if buf.eq(&ALLOW_READ) {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
