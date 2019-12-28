#[test]
fn test_service_parsing() {
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
