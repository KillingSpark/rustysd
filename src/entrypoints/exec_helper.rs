
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ExecHelperConfig {
    pub cmd: String,
    pub args: Vec<String>,

    pub env: Vec<(String, String)>,
}

fn prepare_exec_args(
    cmd_str: &str,
    args_str: &[String],
) -> (std::ffi::CString, Vec<std::ffi::CString>) {
    let cmd = std::ffi::CString::new(cmd_str).unwrap();

    let exec_name = std::path::PathBuf::from(cmd_str);
    let exec_name = exec_name.file_name().unwrap();
    let exec_name: Vec<u8> = exec_name.to_str().unwrap().bytes().collect();
    let exec_name = std::ffi::CString::new(exec_name).unwrap();

    let mut args = Vec::new();
    args.push(exec_name);

    for word in args_str {
        args.push(std::ffi::CString::new(word.as_str()).unwrap());
    }

    (cmd, args)
}

pub fn run_exec_helper() {
    println!("Exec helper trying to read config from stdin");
    let config: ExecHelperConfig = serde_json::from_reader(std::io::stdin()).unwrap();
    println!("Apply config: {:?}", config);
    let (cmd, args) = prepare_exec_args(&config.cmd, &config.args);
    // TODO env and LISTEN_PID env var
    for (k, v) in config.env.iter() {
        std::env::set_var(k, v);
    }

    std::env::set_var("LISTEN_PID", format!("{}", nix::unistd::getpid()));

    eprintln!("EXECV: {:?} {:?}", &cmd, &args);
    nix::unistd::execv(&cmd, &args).unwrap();
}
