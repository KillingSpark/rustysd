use std::io::Write;
use std::path::PathBuf;

pub fn setup_logging(dir: &PathBuf) -> Result<(), String> {
    let lmbrjck_conf = lumberjack_rs::Conf {
        max_age: None,
        max_files: Some(10),
        max_size: 10 * 1024 * 1024,
        log_dir: dir.clone(),
        name_template: "rustysdlog.log".to_owned(),
    };

    let rotating = std::sync::Mutex::new(lumberjack_rs::new(lmbrjck_conf).unwrap());

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Trace)
        .chain(std::io::stdout())
        .chain(fern::Output::call(move |record| {
            let msg = format!("{}\n", record.args());
            let rotating = rotating.lock();
            let mut rotating = rotating.unwrap();
            let result = rotating.write_all(msg.as_str().as_bytes());
            //TODO do something with the result
            let _ = result;
        }))
        .apply()
        .map_err(|e| format!("Error while stting up logger: {}", e))
}
