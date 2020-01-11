pub struct GroupEntry {
    pub name: String,
    pub pw: Option<Vec<u8>>,
    pub gid: nix::unistd::Gid,
}

#[cfg(target_os = "linux")]
pub fn getgrnam(groupname: &str) -> Result<GroupEntry, String> {
    let username_i8 = groupname.bytes().map(|x| x as i8).collect::<Vec<_>>();
    let pointer: *const i8 = username_i8.as_ptr();
    // TODO check errno
    let res = unsafe { libc::getgrnam(pointer) };
    if res.is_null() {
        return Err(format!("No entry found for groupname: {}", groupname));
    }
    let res = unsafe { *res };

    let gid = nix::unistd::Gid::from_raw(res.gr_gid);
    let pw = if !res.gr_passwd.is_null() {
        let mut vec = Vec::new();
        let mut ptr = res.gr_passwd;
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
    Ok(GroupEntry {
        name: groupname.to_string(),
        gid,
        pw,
    })
}

#[cfg(not(target_os = "linux"))]
pub fn getgrnam(username: &str) -> Result<PwEntry, String> {
    Err("getpwnam is not yet implemented for this platform".into())
}
