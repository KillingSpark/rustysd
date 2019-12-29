//! Wait for sockets to activate their respective services

use crate::platform::EventFd;
use crate::units::*;

pub fn wait_for_socket(
    eventfd: EventFd,
    unit_table: ArcMutUnitTable,
) -> Result<Vec<UnitId>, String> {
    let fd_to_srvc_id =
        unit_table
            .read()
            .unwrap()
            .iter()
            .fold(Vec::new(), |mut acc, (id, unit)| {
                let unit_locked = unit.lock().unwrap();
                if let UnitSpecialized::Socket(sock) = &unit_locked.specialized {
                    if !sock.activated {
                        for conf in &sock.sockets {
                            if let Some(sock) = &conf.fd {
                                acc.push((sock.as_raw_fd(), *id));
                            }
                        }
                    }
                }
                acc
            });

    let mut fdset = nix::sys::select::FdSet::new();
    for (fd, _) in &fd_to_srvc_id {
        fdset.insert(*fd);
    }
    fdset.insert(eventfd.read_end());

    let result = nix::sys::select::select(None, Some(&mut fdset), None, None, None);
    match result {
        Ok(_) => {
            let mut activated_ids = Vec::new();
            if fdset.contains(eventfd.read_end()) {
                trace!("Interrupted socketactivation select because the eventfd fired");
                crate::platform::reset_event_fd(eventfd);
                trace!("Reset eventfd value");
            } else {
                for (fd, id) in &fd_to_srvc_id {
                    if fdset.contains(*fd) {
                        activated_ids.push(*id);
                    }
                }
            }
            Ok(activated_ids)
        }
        Err(e) => {
            if let nix::Error::Sys(nix::errno::Errno::EINTR) = e {
                Ok(Vec::new())
            } else {
                Err(format!("Error while selecting: {}", e))
            }
        }
    }
}
