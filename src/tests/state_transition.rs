use crate::runtime_info::*;
use crate::units::Unit;
use std::convert::TryInto;

#[test]
fn test_service_state_transitions() {
    let run_info = std::sync::Arc::new(std::sync::RwLock::new(RuntimeInfo {
        config: crate::config::Config {
            notification_sockets_dir: "./notifications".into(),
            target_unit: "".into(),
            unit_dirs: vec![],
            self_path: std::path::PathBuf::from("./target/debug/rustysd"),
        },
        fd_store: std::sync::RwLock::new(crate::fd_store::FDStore::default()),
        pid_table: std::sync::Mutex::new(PidTable::default()),
        unit_table: UnitTable::default(),
        stdout_eventfd: crate::platform::make_event_fd().unwrap(),
        stderr_eventfd: crate::platform::make_event_fd().unwrap(),
        notification_eventfd: crate::platform::make_event_fd().unwrap(),
        socket_activation_eventfd: crate::platform::make_event_fd().unwrap(),
    }));

    let signals = signal_hook::iterator::Signals::new(&[signal_hook::consts::SIGCHLD]).unwrap();

    let run_info_clone = run_info.clone();
    let _handle = std::thread::spawn(move || {
        // listen on signals from the child processes
        crate::signal_handler::handle_signals(signals, run_info_clone);
    });

    // TODO this can probably done better with a setup function. Need to look into the test framework more.
    // This needs to be used by all tests that need the signal handling, because else the signal handlers interfere.
    successful(run_info.clone());
    failing_startexec(run_info.clone());
}

fn successful(run_info: ArcMutRuntimeInfo) {
    let descr = "This is a description";
    let service_execstart = "/bin/sleep 10";
    let service_execpre = "/bin/true";
    let service_execpost = "/bin/true";
    let service_stop = "/bin/true";
    let service_stoppost = "/bin/true";

    let test_service_str = format!(
        r#"
    [Unit]
    Description = {}
    [Service]
    ExecStart = {}
    ExecStartPre = {}
    ExecStartPost = {}
    ExecStop = {}
    ExecStopPost = {}

    "#,
        descr, service_execstart, service_execpre, service_execpost, service_stop, service_stoppost,
    );

    let parsed_file = crate::units::parse_file(&test_service_str).unwrap();
    let service = crate::units::parse_service(
        parsed_file,
        &std::path::PathBuf::from("/path/to/unitfile.service"),
    )
    .unwrap();
    let unit: Unit = service.try_into().unwrap();

    let unit_id = unit.id.clone();

    run_info
        .write()
        .unwrap()
        .unit_table
        .insert(unit.id.clone(), unit);

    let run_info_locked = run_info.read().unwrap();
    let unit = run_info_locked.unit_table.get(&unit_id).unwrap();

    unit.activate(
        &*run_info.read().unwrap(),
        crate::units::ActivationSource::Regular,
    )
    .unwrap();
    let status = unit.common.status.read().unwrap();

    assert_eq!(
        *status,
        crate::units::UnitStatus::Started(crate::units::StatusStarted::Running)
    );
}

fn failing_startexec(run_info: ArcMutRuntimeInfo) {
    let descr = "This is a description";
    let service_type = "oneshot";
    let service_execstart = "/bin/false";
    let service_execpre = "/bin/true";
    let service_execpost = "/bin/true";
    let service_stop = "/bin/true";
    let service_stoppost = "/bin/true";

    let test_service_str = format!(
        r#"
    [Unit]
    Description = {}
    [Service]
    Type= {}
    ExecStart = {}
    ExecStartPre = {}
    ExecStartPost = {}
    ExecStop = {}
    ExecStopPost = {}

    "#,
        descr,
        service_type,
        service_execstart,
        service_execpre,
        service_execpost,
        service_stop,
        service_stoppost,
    );

    let parsed_file = crate::units::parse_file(&test_service_str).unwrap();
    let service = crate::units::parse_service(
        parsed_file,
        &std::path::PathBuf::from("/path/to/unitfile.service"),
    )
    .unwrap();
    let unit: Unit = service.try_into().unwrap();

    let unit_id = unit.id.clone();

    run_info
        .write()
        .unwrap()
        .unit_table
        .insert(unit.id.clone(), unit);

    let run_info_locked = run_info.read().unwrap();
    let unit = run_info_locked.unit_table.get(&unit_id).unwrap();

    assert!(unit
        .activate(
            &*run_info.read().unwrap(),
            crate::units::ActivationSource::Regular
        )
        .is_err());
    let status = unit.common.status.read().unwrap();

    match &*status {
        crate::units::UnitStatus::Stopped(
            crate::units::StatusStopped::StoppedUnexpected,
            errors,
        ) => {
            if errors.len() != 1 {
                panic!("Wrong amount of errors. Should be 1. Is: {}", errors.len());
            }
            match &errors[0] {
                crate::units::UnitOperationErrorReason::ServiceStartError(
                    crate::services::ServiceErrorReason::StartFailed(
                        crate::services::RunCmdError::BadExitCode(_, _),
                    ),
                ) => {
                    // HAPPY
                }
                other => {
                    panic!(
                        "Wrong error. Should have been ServiceStartError(StartFailed(BadExitCode(_,_))). Is: {:?}",
                        other
                    );
                }
            }
        }
        other => panic!(
            "Wrong status. Should have been StoppedUnexpected. Is: {:?}",
            other
        ),
    };
}
