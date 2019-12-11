use dbus::blocking::Connection;
use dbus::blocking::Proxy;

pub fn wait_for_name(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // TODO make this async
    let conn = Connection::new_system()?;
    let obj = conn.with_proxy("org.freedesktop.DBus", "/", std::time::Duration::from_millis(5000));

    while !name_exists(name, &obj)? {
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    Ok(())
}

fn name_exists(name: &str, obj: &Proxy<&Connection>) -> Result<bool, Box<dyn std::error::Error>> {
    let (names,): (Vec<String>,) = obj.method_call("org.freedesktop.DBus", "ListNames", ())?;
    Ok(names.contains(&name.to_owned()))
}

#[test]
fn test_dbus_wait() {
    wait_for_name("org.freedesktop.DBus").unwrap();
}