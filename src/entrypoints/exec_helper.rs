use std::path::{PathBuf, Path};

use crate::units::PlatformSpecificServiceFields;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ExecHelperConfig {
    pub name: String,

    pub cmd: PathBuf,
    pub args: Vec<String>,

    pub env: Vec<(String, String)>,

    pub group: libc::gid_t,
    pub supplementary_groups: Vec<libc::gid_t>,
    pub user: libc::uid_t,

    pub platform_specific: PlatformSpecificServiceFields,
}

fn prepare_exec_args(
    cmd_str: &Path,
    args_str: &[String],
) -> (std::ffi::CString, Vec<std::ffi::CString>) {
    let cmd = std::ffi::CString::new(cmd_str.to_string_lossy().as_bytes()).unwrap();

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

    nix::unistd::close(libc::STDIN_FILENO).expect("I want to be able to close this fd!");

    if let Err(e) =
        crate::services::fork_os_specific::post_fork_os_specific(&config.platform_specific)
    {
        eprintln!("[FORK_CHILD {}] postfork error: {}", config.name, e);
        std::process::exit(1);
    }

    if nix::unistd::getuid().is_root() {
        match crate::platform::drop_privileges(
            nix::unistd::Gid::from_raw(config.group),
            &config
                .supplementary_groups
                .iter()
                .map(|gid| nix::unistd::Gid::from_raw(*gid))
                .collect(),
            nix::unistd::Uid::from_raw(config.user),
        ) {
            Ok(()) => { /* Happy */ }
            Err(e) => {
                eprintln!(
                    "[EXEC_HELPER {}] could not drop privileges because: {}",
                    config.name, e
                );
                std::process::exit(1);
            }
        }
    }

    let (cmd, args) = prepare_exec_args(&config.cmd, &config.args);

    // setup environment vars
    for (k, v) in config.env.iter() {
        std::env::set_var(k, v);
    }

    std::env::set_var("LISTEN_PID", format!("{}", nix::unistd::getpid()));

    eprintln!("EXECV: {:?} {:?}", &cmd, &args);

    nix::unistd::execv(&cmd, &args).unwrap();
}
