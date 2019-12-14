use std::io::Write;

pub fn setup_logging(conf: &crate::config::LoggingConfig) -> Result<(), String> {
    let mut logger = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Trace);

    if conf.log_to_stdout {
        logger = logger.chain(std::io::stdout());
    }

    if conf.log_to_disk {
        if !conf.log_dir.exists() {
            std::fs::create_dir_all(&conf.log_dir)
                .map_err(|e| format!("Error creating logdir: {}", e))?;
        }
        let lmbrjck_conf = lumberjack_rs::Conf {
            max_age: None,
            max_files: Some(10),
            max_size: 10 * 1024 * 1024,
            log_dir: conf.log_dir.clone(),
            name_template: "rustysdlog.log".to_owned(),
        };

        let rotating = std::sync::Mutex::new(lumberjack_rs::new(lmbrjck_conf).unwrap());

        logger = logger.chain(fern::Output::call(move |record| {
            let msg = format!("{}\n", record.args());
            let rotating = rotating.lock();
            let mut rotating = rotating.unwrap();
            let result = rotating.write_all(msg.as_str().as_bytes());
            //TODO do something with the result
            let _ = result;
        }));
    }

    logger
        .apply()
        .map_err(|e| format!("Error while stting up logger: {}", e))
}
