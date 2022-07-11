fn main() {
    let exec_name = std::env::args().next().expect("could not get executable name from args");
    if exec_name == "exec_helper" {
        rustysd::run_exec_helper();
    } else {
        rustysd::run_service_manager();
    }
}
