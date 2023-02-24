fn main() {
    let exec_name = std::env::args()
        .next()
        .expect("could not get executable name from args");
    if exec_name.ends_with("exec_helper") {
        rustysd::entrypoints::run_exec_helper();
    } else if exec_name.ends_with("rustysd") {
        rustysd::entrypoints::run_service_manager();
    } else {
        eprintln!(
            "Can only start as rustysd or exec_helper. Was: {}",
            exec_name
        )
    }
}
