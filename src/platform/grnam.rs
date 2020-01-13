pub struct GroupEntry {
    pub name: String,
    pub pw: Option<Vec<u8>>,
    pub gid: nix::unistd::Gid,
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
fn make_group_from_libc(groupname: &str, group: &libc::group) -> Result<GroupEntry, String> {
    let gid = nix::unistd::Gid::from_raw(group.gr_gid);
    let pw = if !group.gr_passwd.is_null() {
        let mut vec = Vec::new();
        let mut ptr = group.gr_passwd;
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

#[cfg(target_os = "linux")]
#[allow(dead_code)]
// keep around for a PR to the nix crate
fn getgrnam(groupname: &str) -> Result<GroupEntry, String> {
    let username_i8 = groupname.bytes().map(|x| x as i8).collect::<Vec<_>>();
    let pointer: *const i8 = username_i8.as_ptr();
    // TODO check errno
    let res = unsafe { libc::getgrnam(pointer) };
    if res.is_null() {
        return Err(format!("No entry found for groupname: {}", groupname));
    }
    let res = unsafe { *res };
    make_group_from_libc(groupname, &res)
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
pub fn getgrnam_r(groupname: &str) -> Result<GroupEntry, String> {
    let username_i8 = groupname.bytes().map(|x| x as i8).collect::<Vec<_>>();
    let pointer: *const i8 = username_i8.as_ptr();
    let mut buf_size = 32;
    let mut group: libc::group = libc::group {
        gr_name: std::ptr::null_mut(),
        gr_passwd: std::ptr::null_mut(),
        gr_gid: 0,
        gr_mem: std::ptr::null_mut(),
    };

    let group_ptr = &mut group;
    let group_ptr_ptr = &mut (group_ptr as *mut libc::group);
    loop {
        let mut buf = Vec::with_capacity(buf_size);
        buf.resize(buf_size, 0i8);

        let errno = unsafe {
            libc::getgrnam_r(
                pointer,
                group_ptr,
                buf.as_mut_ptr(),
                buf_size,
                group_ptr_ptr,
            )
        };

        if group_ptr_ptr.is_null() {
            // error case
            if errno == libc::ERANGE {
                // need more bytes in buf
                buf_size = buf_size * 2;
            } else {
                return Err(format!("Error calling getpwnam_r: {}", errno));
            }
        } else {
            // just for safety check this, but this is the happy result
            if (group_ptr as *mut libc::group).eq(&*group_ptr_ptr) {
                return make_group_from_libc(groupname, &*group_ptr);
            } else {
                return Err(format!("The **group ({:?}) should have pointed to the same location as the *group ({:?})", group_ptr_ptr, group_ptr));
            }
        }
    }
}

#[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
pub fn getgrnam_r(_groupname: &str) -> Result<GroupEntry, String> {
    compile_error!("getgrnam_r is not yet implemented for this platform");
}
