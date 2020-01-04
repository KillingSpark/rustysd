use rustysd::control::jsonrpc2::Call;
use serde_json::Value;
use std::io::Write;

fn main() {
    let mut args: Vec<_> = std::env::args().collect();
    let _exec_name = args.remove(0);
    let addr = args.remove(0);
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

    let mut stream = std::net::TcpStream::connect(addr).unwrap();
    println!("Write cmd: {}", str_call);
    stream.write_all(str_call.as_bytes()).unwrap();
    stream.shutdown(std::net::Shutdown::Write).unwrap();
    println!("Wait for response");
    let resp: Value = serde_json::from_reader(&mut stream).unwrap();
    println!("Got response");
    println!("{}", serde_json::to_string_pretty(&resp).unwrap());
}
