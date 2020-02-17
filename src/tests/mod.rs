#[test]
fn test_service_parsing() {
    let descr = "This is a description";
    let unit_before1 = "unit_before2";
    let unit_before2 = "unit_before1";
    let unit_after1 = "unit_after1";
    let unit_after2 = "unit_after2,unit_after3";

    let install_required_by = "install_req_by";
    let install_wanted_by = "install_wanted_by";

    let service_execstart = "/path/to/startbin arg1 arg2 arg3";
    let service_execpre = "--/path/to/startprebin arg1 arg2 arg3";
    let service_execpost = "/path/to/startpostbin arg1 arg2 arg3";
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
    ExecStart = {}
    ExecStartPre = {}
    ExecStartPost = {}
    ExecStop = {}
    Sockets = {}

    "#,
        descr,
        unit_before1,
        unit_before2,
        unit_after1,
        unit_after2,
        install_required_by,
        install_wanted_by,
        service_execstart,
        service_execpre,
        service_execpost,
        service_stop,
        service_sockets,
    );

    let parsed_file = crate::units::parse_file(&test_service_str).unwrap();
    let service = crate::units::parse_service(
        parsed_file,
        &std::path::PathBuf::from("/path/to/unitfile.service"),
        crate::units::UnitId(crate::units::UnitIdKind::Service, 10),
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
        vec![
            unit_after1.to_owned(),
            "unit_after2".to_owned(),
            "unit_after3".to_owned()
        ]
    );

    if let Some(conf) = service.install.install_config {
        assert_eq!(conf.required_by, vec![install_required_by.to_owned()]);
        assert_eq!(conf.wanted_by, vec![install_wanted_by.to_owned()]);
    } else {
        panic!("No install config found, but there should be one");
    }
    if let crate::units::UnitSpecialized::Service(srvc) = service.specialized {
        assert_eq!(
            srvc.service_config.exec,
            crate::units::Commandline {
                cmd: "/path/to/startbin".into(),
                args: vec!["arg1".into(), "arg2".into(), "arg3".into()],
                prefixes: vec![],
            }
        );
        assert_eq!(
            srvc.service_config.startpre,
            vec![crate::units::Commandline {
                cmd: "/path/to/startprebin".into(),
                args: vec!["arg1".into(), "arg2".into(), "arg3".into()],
                prefixes: vec![
                    crate::units::CommandlinePrefix::Minus,
                    crate::units::CommandlinePrefix::Minus,
                ],
            }]
        );
        assert_eq!(
            srvc.service_config.startpost,
            vec![crate::units::Commandline {
                cmd: "/path/to/startpostbin".into(),
                args: vec!["arg1".into(), "arg2".into(), "arg3".into()],
                prefixes: vec![],
            }]
        );
        assert_eq!(
            srvc.service_config.stop,
            vec![crate::units::Commandline {
                cmd: "/path/to/stopbin".into(),
                args: vec!["arg1".into(), "arg2".into(), "arg3".into()],
                prefixes: vec![],
            }]
        );
        assert_eq!(
            srvc.service_config.sockets,
            vec!["socket_name1".to_owned(), "socket_name2".to_owned()]
        );
    } else {
        panic!("Not a service, but it should be");
    }
}

#[test]
fn test_socket_parsing() {
    let descr = "This is a description";
    let unit_before1 = "unit_before2";
    let unit_before2 = "unit_before1";
    let unit_after1 = "unit_after1";
    let unit_after2 = "unit_after2,unit_after3";

    let install_required_by = "install_req_by";
    let install_wanted_by = "install_wanted_by";

    let socket_fdname = "SocketyMcSockface";
    let socket_ipv4 = "127.0.0.1:8080";
    let socket_ipv6 = "[fe81::]:8080";
    let socket_unix = "/path/to/socket";
    let socket_service = "other_name";

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
    
    [Socket]
    ListenStream = {}
    ListenStream = {}
    ListenStream = {}

    ListenDatagram = {}
    ListenDatagram = {}
    ListenDatagram = {}

    ListenSequentialPacket = {}
    ListenFifo = {}
    Service= {}
    FileDescriptorName= {}

    "#,
        descr,
        unit_before1,
        unit_before2,
        unit_after1,
        unit_after2,
        install_required_by,
        install_wanted_by,
        socket_ipv4,
        socket_ipv6,
        socket_unix,
        socket_ipv4,
        socket_ipv6,
        socket_unix,
        socket_unix,
        socket_unix,
        socket_service,
        socket_fdname,
    );

    let parsed_file = crate::units::parse_file(&test_service_str).unwrap();
    let socket_unit = crate::units::parse_socket(
        parsed_file,
        &std::path::PathBuf::from("/path/to/unitfile.socket"),
        crate::units::UnitId(crate::units::UnitIdKind::Socket, 10),
    )
    .unwrap();

    // check all the values

    assert_eq!(socket_unit.conf.description, descr);
    assert_eq!(
        socket_unit.conf.before,
        vec![unit_before1.to_owned(), unit_before2.to_owned()]
    );
    assert_eq!(
        socket_unit.conf.after,
        vec![
            unit_after1.to_owned(),
            "unit_after2".to_owned(),
            "unit_after3".to_owned()
        ]
    );

    if let Some(conf) = socket_unit.install.install_config {
        assert_eq!(conf.required_by, vec![install_required_by.to_owned()]);
        assert_eq!(conf.wanted_by, vec![install_wanted_by.to_owned()]);
    } else {
        panic!("No install config found, but there should be one");
    }
    if let crate::units::UnitSpecialized::Socket(sock) = socket_unit.specialized {
        if sock.sockets.len() == 8 {
            // streaming sockets
            if let crate::sockets::SpecializedSocketConfig::TcpSocket(tcpconf) =
                &sock.sockets[0].specialized
            {
                if !tcpconf.addr.is_ipv4() {
                    panic!("Should have been an ipv4 address but wasnt");
                }
            } else {
                panic!("Sockets[0] should have been a tcp socket, but wasnt");
            }
            if let crate::sockets::SpecializedSocketConfig::TcpSocket(tcpconf) =
                &sock.sockets[1].specialized
            {
                if !tcpconf.addr.is_ipv6() {
                    panic!("Should have been an ipv6 address but wasnt");
                }
            } else {
                panic!("Sockets[1] should have been a tcp socket, but wasnt");
            }
            if let crate::sockets::SpecializedSocketConfig::UnixSocket(
                crate::sockets::UnixSocketConfig::Stream(addr),
            ) = &sock.sockets[2].specialized
            {
                assert_eq!(addr, socket_unix);
            } else {
                panic!("Sockets[2] should have been a streaming unix socket, but wasnt");
            }

            // Datagram sockets
            if let crate::sockets::SpecializedSocketConfig::UdpSocket(tcpconf) =
                &sock.sockets[3].specialized
            {
                if !tcpconf.addr.is_ipv4() {
                    panic!("Should have been an ipv4 address but wasnt");
                }
            } else {
                panic!("Sockets[3] should have been a udp socket, but wasnt");
            }
            if let crate::sockets::SpecializedSocketConfig::UdpSocket(tcpconf) =
                &sock.sockets[4].specialized
            {
                if !tcpconf.addr.is_ipv6() {
                    panic!("Should have been an ipv6 address but wasnt");
                }
            } else {
                panic!("Sockets[4] should have been a udp socket, but wasnt");
            }
            if let crate::sockets::SpecializedSocketConfig::UnixSocket(
                crate::sockets::UnixSocketConfig::Datagram(addr),
            ) = &sock.sockets[5].specialized
            {
                assert_eq!(addr, socket_unix);
            } else {
                panic!("Sockets[5] should have been a datagram unix socket, but wasnt");
            }

            // SeqPacket socket
            if let crate::sockets::SpecializedSocketConfig::UnixSocket(
                crate::sockets::UnixSocketConfig::Sequential(addr),
            ) = &sock.sockets[6].specialized
            {
                assert_eq!(addr, socket_unix);
            } else {
                panic!("Sockets[6] should have been a sequential packet unix socket, but wasnt");
            }
            // SeqPacket socket
            if let crate::sockets::SpecializedSocketConfig::Fifo(fifoconf) =
                &sock.sockets[7].specialized
            {
                assert_eq!(fifoconf.path, std::path::PathBuf::from(socket_unix));
            } else {
                panic!("Sockets[6] should have been a sequential packet unix socket, but wasnt");
            }
        } else {
            panic!("Not enough sockets parsed");
        }
    } else {
        panic!("Not a service, but it should be");
    }
}

#[test]
fn test_unit_ordering() {
    let target1_str = format!(
        "
    [Unit]
    Description = {}
    Before = {}
    
    [Install]
    RequiredBy = {}
    ",
        "Target", "2.target", "2.target",
    );

    let parsed_file = crate::units::parse_file(&target1_str).unwrap();
    let target1_unit = crate::units::parse_target(
        parsed_file,
        &std::path::PathBuf::from("/path/to/1.target"),
        crate::units::UnitId(crate::units::UnitIdKind::Socket, 1),
    )
    .unwrap();

    let target2_str = format!(
        "
    [Unit]
    Description = {}
    After = {}

    [Install]
    RequiredBy = {}
    ",
        "Target", "1.target", "3.target",
    );

    let parsed_file = crate::units::parse_file(&target2_str).unwrap();
    let target2_unit = crate::units::parse_target(
        parsed_file,
        &std::path::PathBuf::from("/path/to/2.target"),
        crate::units::UnitId(crate::units::UnitIdKind::Socket, 2),
    )
    .unwrap();

    let target3_str = format!(
        "
    [Unit]
    Description = {}
    After = {}
    
    ",
        "Target", "1.target"
    );

    let parsed_file = crate::units::parse_file(&target3_str).unwrap();
    let target3_unit = crate::units::parse_target(
        parsed_file,
        &std::path::PathBuf::from("/path/to/3.target"),
        crate::units::UnitId(crate::units::UnitIdKind::Socket, 3),
    )
    .unwrap();

    let mut unit_table = std::collections::HashMap::new();
    let id1 = target1_unit.id;
    let id2 = target2_unit.id;
    let id3 = target3_unit.id;
    unit_table.insert(target1_unit.id, target1_unit);
    unit_table.insert(target2_unit.id, target2_unit);
    unit_table.insert(target3_unit.id, target3_unit);

    crate::units::fill_dependencies(&mut unit_table);
    crate::units::add_implicit_before_after(&mut unit_table);
    unit_table
        .values_mut()
        .for_each(|unit| unit.dedup_dependencies());
    crate::units::sanity_check_dependencies(&unit_table).unwrap();

    unit_table
        .values()
        .for_each(|unit| println!("{} {:?}", unit.id, unit.install));

    // before/after 1.target
    assert!(unit_table.get(&id1).unwrap().install.after.is_empty());
    assert!(unit_table.get(&id1).unwrap().install.before.len() == 2);
    assert!(unit_table.get(&id1).unwrap().install.before.contains(&id2));
    assert!(unit_table.get(&id1).unwrap().install.before.contains(&id3));

    // before/after 2.target
    assert_eq!(unit_table.get(&id2).unwrap().install.before.len(), 1);
    assert!(unit_table.get(&id2).unwrap().install.before.contains(&id3));
    assert_eq!(unit_table.get(&id2).unwrap().install.after.len(), 1);
    assert!(unit_table.get(&id2).unwrap().install.after.contains(&id1));

    // before/after 3.target
    assert!(unit_table.get(&id3).unwrap().install.before.is_empty());
    assert!(unit_table.get(&id3).unwrap().install.after.len() == 2);
    assert!(unit_table.get(&id3).unwrap().install.after.contains(&id2));
    assert!(unit_table.get(&id3).unwrap().install.after.contains(&id1));
}

#[test]
fn test_circle() {
    let target1_str = format!(
        "
    [Unit]
    Description = {}
    After = {}
    ",
        "Target", "3.target"
    );

    let parsed_file = crate::units::parse_file(&target1_str).unwrap();
    let target1_unit = crate::units::parse_target(
        parsed_file,
        &std::path::PathBuf::from("/path/to/1.target"),
        crate::units::UnitId(crate::units::UnitIdKind::Socket, 1),
    )
    .unwrap();

    let target2_str = format!(
        "
    [Unit]
    Description = {}
    After = {}
    ",
        "Target", "1.target"
    );

    let parsed_file = crate::units::parse_file(&target2_str).unwrap();
    let target2_unit = crate::units::parse_target(
        parsed_file,
        &std::path::PathBuf::from("/path/to/2.target"),
        crate::units::UnitId(crate::units::UnitIdKind::Socket, 2),
    )
    .unwrap();

    let target3_str = format!(
        "
    [Unit]
    Description = {}
    After = {}
    ",
        "Target", "2.target"
    );

    let parsed_file = crate::units::parse_file(&target3_str).unwrap();
    let target3_unit = crate::units::parse_target(
        parsed_file,
        &std::path::PathBuf::from("/path/to/3.target"),
        crate::units::UnitId(crate::units::UnitIdKind::Socket, 3),
    )
    .unwrap();

    let mut unit_table = std::collections::HashMap::new();
    let target1_id = target1_unit.id;
    let target2_id = target2_unit.id;
    let target3_id = target3_unit.id;
    unit_table.insert(target1_unit.id, target1_unit);
    unit_table.insert(target2_unit.id, target2_unit);
    unit_table.insert(target3_unit.id, target3_unit);

    crate::units::fill_dependencies(&mut unit_table);
    crate::units::add_implicit_before_after(&mut unit_table);
    unit_table
        .values_mut()
        .for_each(|unit| unit.dedup_dependencies());

    if let Err(crate::units::SanityCheckError::CirclesFound(circles)) =
        crate::units::sanity_check_dependencies(&unit_table)
    {
        if circles.len() == 1 {
            let circle = &circles[0];
            assert_eq!(circle.len(), 3);
            assert!(circle.contains(&target1_id));
            assert!(circle.contains(&target2_id));
            assert!(circle.contains(&target3_id));
        } else {
            panic!("more than one circle found but there is only one");
        }
    } else {
        panic!("No circle found but there is one");
    }
}
