pub struct PwEntry {
    pub name: String,
    pub pw: Option<Vec<u8>>,
    pub uid: nix::unistd::Uid,
    pub gid: nix::unistd::Gid,
}

// TODO PR to nix
#[cfg(target_os = "linux")]
pub fn getpwnam(username: &str) -> Result<PwEntry, String> {
    let username_i8 = username.bytes().map(|x| x as i8).collect::<Vec<_>>();
    let pointer: *const i8 = username_i8.as_ptr();
    // TODO check errno
    let res = unsafe { libc::getpwnam(pointer) };
    if res.is_null() {
        return Err(format!("No entry found for username: {}", username));
    }
    let res = unsafe { *res };

    let uid = nix::unistd::Uid::from_raw(res.pw_uid);
    let gid = nix::unistd::Gid::from_raw(res.pw_gid);
    let pw = if !res.pw_passwd.is_null() {
        let mut vec = Vec::new();
        let mut ptr = res.pw_passwd;
        loop {
            let byte = unsafe { *ptr } as u8;
            if byte == b'\0' {
                break;
            } else {
                vec.push(byte);
            }
            unsafe { ptr = ptr.add(1) };
        }
        Some(vec)
    } else {
        None
    };
    Ok(PwEntry {
        name: username.to_string(),
        uid,
        gid,
        pw,
    })
}

#[cfg(not(target_os = "linux"))]
pub fn getpwnam(username: &str) -> Result<PwEntry, String> {
    Err("getpwnam is not yet implemented for this platform".into())
}
