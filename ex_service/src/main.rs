use rservice::server::{srv_fn, ServiceServer};
use std::sync::{Arc, Mutex};

fn append_hello(_: Arc<Mutex<Box<()>>>, mut args: Vec<String>) -> String {
    if let Some(first) = args.get_mut(0) {
        format!("hello {}", first)
    } else {
        format!("hello world")
    }
}

fn ping(_: Arc<Mutex<Box<()>>>, _: Vec<String>) -> String {
    "pong".to_string()
}

fn main() {
    let srv = ServiceServer::new("example",
        [
            srv_fn!(append_hello),
            srv_fn!(ping),
        ], ()).expect("Failed to register service");
    srv.run();
}
