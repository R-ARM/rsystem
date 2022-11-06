use ozone::{init, Config};
use rservice::server::{srv_fn, ServiceServer};
use std::{
    thread,
    sync::{Arc, Mutex},
};

fn launch_service(_: Arc<Mutex<Box<()>>>, mut args: Vec<String>) -> String {
    let Some(service) = args.get(0) else {
        return String::from("ERR");
    };

    String::from("OK")
}

fn start_service() {
    let srv = ServiceServer::new("init",
        [
            srv_fn!(launch_service),
        ]).expect("Failed to register service");
    srv.run();
}

fn main() {
    let conf = Config::new()
        .mount_sys(true);
    init(&conf).expect("Basic init failed!");

    thread::spawn(|| start_service());

    println!("Hello, world!");
    loop {}
}
