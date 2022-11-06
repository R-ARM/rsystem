use rservice::client;
use std::sync::{Arc, Mutex};

fn main() {
    let argv: Vec<String> = std::env::args().collect();
    let method = &argv[1];
    let args = if argv.len() > 1 {
        &argv[2..]
    } else {
        &[]
    };

    let mut srv = client::get_service("example").unwrap();

    let resp = srv.call(method, args).unwrap();

    println!("Response: {}", resp);
}
