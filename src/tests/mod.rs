#[test]
fn test_some_stuff() {
    let descr = "This is a description";
    let unit_before1 = "unit_before2";
    let unit_before2 = "unit_before1";
    let unit_after1 = "unit_after1";
    let unit_after2 = "unit_after2,unit_after3";

    let install_required_by = "install_req_by";
    let install_wanted_by = "install_wanted_by";

    let service_exec = "/path/to/startbin arg1 arg2 arg3";
    let service_stop = "/path/to/stopbin arg1 arg2 arg3";
    let service_sockets = "socket_name1,socket_name2";

    let test_service_str = format!(
        r#"
    [Unit]
    Description = {}
    Before = {}
    Before = {}
    After = {}
    After = {}
    
    [Install]
    RequiredBy = {}
    WantedBy = {}
    
    [Service]
    Exec = {}
    Stop = {}
    Sockets = {}

    "#,
        descr,
        unit_before1,
        unit_before2,
        unit_after1,
        unit_after2,
        install_required_by,
        install_wanted_by,
        service_exec,
        service_stop,
        service_sockets,
    );

    let parsed_file = crate::units::parse_file(&test_service_str).unwrap();
    let service = crate::units::parse_service(
        parsed_file,
        &std::path::PathBuf::from("/path/to/unitfile.service"),
        10,
    )
    .unwrap();

    // check all the values

    assert_eq!(service.conf.description, descr);
    assert_eq!(
        service.conf.before,
        vec![unit_before1.to_owned(), unit_before2.to_owned()]
    );
    assert_eq!(
        service.conf.after,
        vec![unit_after1.to_owned(), "unit_after2".to_owned(), "unit_after3".to_owned()]
    );

    if let Some(conf) = service.install.install_config {
        assert_eq!(conf.required_by, vec![install_required_by.to_owned()]);
        assert_eq!(conf.wanted_by, vec![install_wanted_by.to_owned()]);
    } else {
        panic!("No install config found, but there should be one");
    }
    if let crate::units::UnitSpecialized::Service(srvc) = service.specialized {
        if let Some(conf) = srvc.service_config {
            assert_eq!(conf.exec, service_exec);
            assert_eq!(conf.stop, service_stop);
            assert_eq!(
                conf.sockets,
                vec!["socket_name1".to_owned(), "socket_name2".to_owned()]
            );
        } else {
            panic!("No service config found, but there should be one");
        }
    } else {
        panic!("Not a service, but it should be");
    }
}
