//! Systemd has the feature to wait on services that have type dbus. This means it waits until a speicifc name has been grabbed on the bus.
//! This is made optional here to not have a hard dependency on libdbus.

#[cfg(feature = "dbus_support")]
pub use dbus_support::*;

#[cfg(not(feature = "dbus_support"))]
pub use no_dbus_support::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum WaitResult {
    Ok,
    Timedout,
}

#[cfg(not(feature = "dbus_support"))]
mod no_dbus_support {

    use super::WaitResult;
    use std::error::Error;

    pub fn wait_for_name_system_bus(
        _name: &str,
        _timeout: Option<std::time::Duration>,
    ) -> Result<WaitResult, Box<dyn Error>> {
        Err("Dbus is not supported in this build")?;

        // remove warinings about unused code in the enum
        let _ = WaitResult::Ok;
        let _ = WaitResult::Timedout;
        unreachable!();
    }

    // just used for testing
    #[allow(dead_code)]
    pub fn wait_for_name_session_bus(
        _name: &str,
        _timeout: Option<std::time::Duration>,
    ) -> Result<WaitResult, Box<dyn Error>> {
        Err("Dbus is not supported in this build")?;
        unreachable!();
    }
}

#[cfg(feature = "dbus_support")]
mod dbus_support {

    extern crate dbus;
    use super::WaitResult;
    use dbus::arg;
    use dbus::blocking::Connection;
    use dbus::blocking::Proxy;
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct NameOwnerChangedHappend {
        pub sender: Vec<String>,
    }

    impl arg::AppendAll for NameOwnerChangedHappend {
        fn append(&self, i: &mut arg::IterAppend) {
            arg::RefArg::append(&self.sender, i);
        }
    }

    impl arg::ReadAll for NameOwnerChangedHappend {
        fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
            let mut vec: Vec<String> = Vec::new();
            loop {
                match i.read() {
                    Ok(s) => vec.push(s),
                    Err(_) => break,
                }
            }
            Ok(NameOwnerChangedHappend { sender: vec })
        }
    }

    impl dbus::message::SignalArgs for NameOwnerChangedHappend {
        const NAME: &'static str = "NameOwnerChanged";
        const INTERFACE: &'static str = "org.freedesktop.DBus";
    }

    pub fn wait_for_name_system_bus(
        name: &str,
        timeout: Option<std::time::Duration>,
    ) -> Result<WaitResult, Box<dyn std::error::Error>> {
        let conn = Connection::new_system()?;
        wait_for_name(name, conn, timeout)
    }

    // just used for testing
    #[allow(dead_code)]
    pub fn wait_for_name_session_bus(
        name: &str,
        timeout: Option<std::time::Duration>,
    ) -> Result<WaitResult, Box<dyn std::error::Error>> {
        let conn = Connection::new_session()?;
        wait_for_name(name, conn, timeout)
    }

    fn wait_for_name(
        name: &str,
        mut conn: Connection,
        timeout: Option<std::time::Duration>,
    ) -> Result<WaitResult, Box<dyn std::error::Error>> {
        let obj = conn.with_proxy(
            "org.freedesktop.DBus",
            "/org/freedesktop/DBus",
            std::time::Duration::from_millis(5000),
        );

        // shortcut if name already exists
        if name_exists(name, &obj)? {
            return Ok(WaitResult::Ok);
        }

        let stoparc = Arc::new(Mutex::new(false));
        let stoparc_cb = stoparc.clone();

        let name = name.to_owned();
        let _id = obj.match_signal(move |h: NameOwnerChangedHappend, _: &Connection| {
            if h.sender[0] == name {
                (*stoparc_cb.lock().unwrap()) = true;
            }
            true
        });

        let start = std::time::Instant::now();
        loop {
            if let Some(timeout) = timeout {
                if *stoparc.lock().unwrap() || start.elapsed() >= timeout {
                    break;
                }
            }
            let max_wait = if let Some(timeout) = timeout {
                timeout - start.elapsed()
            } else {
                std::time::Duration::from_millis(500)
            };
            // TODO PR to dbus-rs so it takes an Option<Duration>
            conn.process(max_wait)?;
        }
        if let Some(timeout) = timeout {
            if start.elapsed() >= timeout {
                Ok(WaitResult::Timedout)
            } else {
                Ok(WaitResult::Ok)
            }
        } else {
            Ok(WaitResult::Ok)
        }
    }

    fn name_exists(
        name: &str,
        obj: &Proxy<&Connection>,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let (names,): (Vec<String>,) = obj.method_call("org.freedesktop.DBus", "ListNames", ())?;
        Ok(names.contains(&name.to_owned()))
    }

    #[test]
    fn test_dbus_wait() {
        let name = "This.Is.A.Test.Name".to_owned();
        let name2 = name.clone();

        std::thread::spawn(move || {
            // wait so the other thread has time to start waiting for the signal
            std::thread::sleep(std::time::Duration::from_secs(3));

            // request name
            let conn = Connection::new_session().unwrap();
            let _reply = conn.request_name(&name2, true, true, true).unwrap();
        });

        // wait for the name to be requested
        match wait_for_name_session_bus(&name, std::time::Duration::from_millis(10_000)).unwrap() {
            WaitResult::Ok => {
                println!("SUCCESS!!");
            }
            WaitResult::Timedout => {
                panic!("FAILED!!");
            }
        }

        // release the name after we are done
        let conn = Connection::new_session().unwrap();
        let _reply = conn.release_name(&name).unwrap();
    }
}
