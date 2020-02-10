# Pid handling
This document gives a overview of how information about pids is passed around in rustysd

In the center of this is the pid_table of the RuntimeInfo struct

## Starting new processes
If a new process is started an entry in the pid_table is created.
1. For Services it is either Service(service_id, pid) or OneshotService(service_id)
1. For ExecStartPre/-Post and ExecStop(-Post) it is Helper(service_id)
These entries tie service-ids and pids together so rustysd knows which service a process belonged to

IMPORTANT: To avoid races for very short lifed processes the pid_table has to be locked before starting new processes. Failing to do that could 
result in the signal listening thread calling the exit_handler while there is not yet an entry for that pid.
```
let spawn_result = {
    let mut pid_table_locked = pid_table.lock().unwrap();
    let res = cmd.spawn();
    if let Ok(child) = &res {
        pid_table_locked.insert(
            nix::unistd::Pid::from_raw(child.id() as i32),
            PidEntry::Helper(id, name.to_string()),
        );
    }
    res
};
```

## Exiting processes
If a process exits services::service_exit_handler::service_exit_handler(...) gets called. This routine does:
1. Check which kind of process exited (Helper/Service process)
1. If it was a helper remove the entry and put in a new entry HelperExited(exit_code) and exit
1. If it was a oneshot service remove the entry and put in a new entry OneshotExited(exit_code) and exit
1. If it was a service check if it was in the 'Started' state (if not it was killed by another rustysd mechanism)
1. Either reactivate the service or cleanup the service and all dependent services according to its config

## Waiting for Helpers
Since we already have a thread calling waitpid() we cant use it to reliably wait for helpers to exit. To work around this
the exit_handler places a HelperExited entry in the pid_table once a helper process exited. 

Currently this means waiting for helpers (and also oneshot services) to exit is busy-waiting on the pid_table entry of
this pid to change from Helper(...) to HelperExited(...). This could be imporved by using a condition variable that 
notifies when an entry in the pid_table  has changed.