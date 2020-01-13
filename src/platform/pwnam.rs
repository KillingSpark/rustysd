pub struct PwEntry {
    pub name: String,
    pub pw: Option<Vec<u8>>,
    pub uid: nix::unistd::Uid,
    pub gid: nix::unistd::Gid,
}

fn make_user_from_libc(username: &str, user: &libc::passwd) -> Result<PwEntry, String> {
    let uid = nix::unistd::Uid::from_raw(user.pw_uid);
    let gid = nix::unistd::Gid::from_raw(user.pw_gid);
    let pw = if !user.pw_passwd.is_null() {
        let mut vec = Vec::new();
        let mut ptr = user.pw_passwd;
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

// TODO PR to nix
#[cfg(target_os = "linux")]
#[allow(dead_code)]
// keep around for a PR to the nix crate
fn getpwnam(username: &str) -> Result<PwEntry, String> {
    let username_i8 = username.bytes().map(|x| x as i8).collect::<Vec<_>>();
    let pointer: *const i8 = username_i8.as_ptr();
    // TODO check errno
    let res = unsafe { libc::getpwnam(pointer) };
    if res.is_null() {
        return Err(format!("No entry found for username: {}", username));
    }
    let res = unsafe { *res };

    make_user_from_libc(username, &res)
}

#[cfg(target_os = "linux")]
fn make_new_pw() -> libc::passwd {
    libc::passwd {
        pw_name: std::ptr::null_mut(),
        pw_passwd: std::ptr::null_mut(),
        pw_uid: 0,
        pw_gid: 0,
        pw_gecos: std::ptr::null_mut(),
        pw_dir: std::ptr::null_mut(),
        pw_shell: std::ptr::null_mut(),
    }
}

#[cfg(target_os = "freebsd")]
fn make_new_pw() -> libc::passwd {
    libc::passwd {
        pw_name: std::ptr::null_mut(),
        pw_passwd: std::ptr::null_mut(),
        pw_uid: 0,
        pw_gid: 0,
        pw_change: 0,
        pw_class: std::ptr::null_mut(),
        pw_gecos: std::ptr::null_mut(),
        pw_dir: std::ptr::null_mut(),
        pw_shell: std::ptr::null_mut(),
        pw_expire: 0,
        pw_fields: 0,
    }
}

#[cfg(any(target_os = "freebsd", target_os = "linux"))]
pub fn getpwnam_r(username: &str) -> Result<PwEntry, String> {
    let username_i8 = username.bytes().map(|x| x as i8).collect::<Vec<_>>();
    let pointer: *const i8 = username_i8.as_ptr();
    let mut buf_size = 32;
    let mut user = make_new_pw();
    let user_ptr = &mut user;
    let user_ptr_ptr = &mut (user_ptr as *mut libc::passwd);
    loop {
        let mut buf = Vec::with_capacity(buf_size);
        buf.resize(buf_size, 0i8);

        let errno = unsafe {
            libc::getpwnam_r(pointer, user_ptr, buf.as_mut_ptr(), buf_size, user_ptr_ptr)
        };

        if user_ptr_ptr.is_null() {
            // error case
            if errno == libc::ERANGE {
                // need more bytes in buf
                buf_size = buf_size * 2;
            } else {
                return Err(format!("Error calling getgrnam_r: {}", errno));
            }
        } else {
            // just for safety check this, but this is the happy result
            if (user_ptr as *mut libc::passwd).eq(&*user_ptr_ptr) {
                return make_user_from_libc(username, &*user_ptr);
            } else {
                return Err(format!("The **user ({:?}) should have pointed to the same location as the *user ({:?})", user_ptr_ptr, user_ptr));
            }
        }
    }
}

#[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
pub fn getpwnam_r(_username: &str) -> Result<PwEntry, String> {
    compile_error!("getpwnam_r is not yet implemented for this platform");
}
