//! RSDCTL
//! This is a utility to pack cli args into the jsonrpc2 format and send them to rustysd.
//! It will read the answer and pretty print it. In the future there might be a more sophisticated client.
//! For now this should suffice.
//!
//! Note that this doesn't even check for the correctness of commands and there args, this is done by the main binary "rustysd"

use rustysd::control::jsonrpc2::Call;
use serde_json::Value;
use std::io::Write;

fn main() {
    let mut args: Vec<_> = std::env::args().collect();
    let _exec_name = args.remove(0);
    if args[0] == "--help" {
        println!("
        This is a utility to pack cli args into the jsonrpc2 format and send them to rustysd.
        It will read the answer and pretty print it. In the future there might be a more sophisticated client.
        For now this should suffice.
        
        Usage:
            rsdctl <ip-addr:port> <command> [args]
        
        Example:
            rsdctl 0.0.0.0:8080 restart test.service
        ");
        return;
    }

    let addr = if std::env::var("RSDCTL_ADDR").is_ok() {
        std::env::var("RSDCTL_ADDR").unwrap()
    } else {
        args.remove(0)
    };
    let args = args;

    let params = if args.len() == 2 {
        Some(Value::String(args[1].clone()))
    } else if args.len() > 1 {
        Some({
            args[1..]
                .iter()
                .cloned()
                .map(|s| Value::String(s))
                .collect()
        })
    } else {
        None
    };

    let call = Call {
        method: args[0].clone(),
        params: params,
        id: None,
    };
    let str_call = serde_json::to_string(&call.to_json()).unwrap();

    if addr.starts_with('/') {
        let mut stream = std::os::unix::net::UnixStream::connect(&addr).unwrap();
        println!("Write cmd: {}", str_call);
        stream.write_all(str_call.as_bytes()).unwrap();
        stream.shutdown(std::net::Shutdown::Write).unwrap();
        println!("Wait for response");
        let resp: Value = serde_json::from_reader(&mut stream).unwrap();
        println!("Got response");
        println!("{}", serde_json::to_string_pretty(&resp).unwrap());
    } else {
        let mut stream = std::net::TcpStream::connect(addr).unwrap();
        println!("Write cmd: {}", str_call);
        stream.write_all(str_call.as_bytes()).unwrap();
        stream.shutdown(std::net::Shutdown::Write).unwrap();
        println!("Wait for response");
        let resp: Value = serde_json::from_reader(&mut stream).unwrap();
        println!("Got response");
        println!("{}", serde_json::to_string_pretty(&resp).unwrap());
    };
}
