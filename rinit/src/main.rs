use ozone::{init, Config};
use rservice::server::{srv_fn, ServiceServer};
use std::{
    thread,
    sync::{Arc, Mutex},
    path::PathBuf,
    process::Command,
    fs,
};

fn launch_service(_: Arc<Mutex<Box<()>>>, args: Vec<String>) -> String {
    let Some(service) = args.get(0) else {
        return String::from("ERR");
    };

    let srv_executable: PathBuf = ["/bin/srv/", service].into_iter().collect();
    if !srv_executable.exists() {
        return String::from("ERR");
    }

    // check if a program with name "service" already runs, if yes pretend we just launched it
    let is_launched = fs::read_dir("/proc").expect("Failed to open /proc")
        .filter_map(|v| v.ok())
        .filter(|v| v.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter_map(|v| {
            let mut path = v.path();
            path.push("cmdline");
            fs::read_to_string(path).ok()
        }).any(|v| v.contains(service));
    
    if is_launched {
        return String::from("OK")
    }

    let mut child = Command::new(srv_executable)
        .spawn()
        .expect("Failed to spawn service");

    thread::spawn(move || child.wait().expect("Failed to wait on a child"));

    String::from("OK")
}

fn start_rservice() {
    let srv = ServiceServer::new("init",
        [
            srv_fn!(launch_service),
        ], ()).expect("Failed to register service");
    srv.run();
}

fn main() {
    let conf = Config::new()
        .mount_sys(true);
    init(&conf).expect("Basic init failed!");

    thread::spawn(|| start_rservice());

    let target = fs::read_to_string("/etc/autorun").unwrap_or("/doesnt/exist".to_string());
    let target_path = PathBuf::from(target);
    if target_path.exists() {
        let child = Command::new(target_path)
            .spawn()
            .expect("Failed to spawn autorun binary");
    }

    println!("Hello, world!");
    loop {}
}
