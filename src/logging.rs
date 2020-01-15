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
        unimplemented!("Logging to disk is currently not supported. Pipe the stdout logs to your preferred logging solution");
    }

    logger
        .apply()
        .map_err(|e| format!("Error while stting up logger: {}", e))
}
