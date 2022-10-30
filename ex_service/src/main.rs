use rservice::server::ServiceServer;
use std::sync::{Arc, Mutex};

fn append_hello(_: Arc<Mutex<Box<()>>>, mut args: Vec<String>) -> String {
    if let Some(first) = args.get_mut(0) {
        format!("hello {}", first)
    } else {
        format!("hello world")
    }
}

fn main() {
    let srv = ServiceServer::new("example",
        [
            ("append_hello", append_hello as fn(Arc<Mutex<Box<_>>>, Vec<_>) -> String),
        ], ()).expect("Failed to register service");
    srv.run();
}
