use std::{
    collections::HashSet,
    path::PathBuf,
    os::unix::net::UnixStream,
    io::{
        Read,
        Write,
    },
};

pub struct ServiceClient {
    stream: UnixStream,
}

impl ServiceClient {
    fn from_socket(mut input: UnixStream) -> Option<Self> {
        Some(Self {
            stream: input,
        })
    }
    pub fn call(&mut self, method: impl ToString, args: impl IntoIterator<Item = impl ToString>) -> Option<String> {
        let mut tmp: Vec<String> = Vec::new();
        tmp.push(method.to_string());
        tmp.extend(args.into_iter().map(|v| v.to_string()));
        let mut full_call: String = tmp.into_iter()
            .map(|v| format!("{}:", v))
            .collect();
        full_call.pop();
        full_call.push('\n');

        self.stream.write_all(full_call.as_str().as_bytes()).ok()?;

        let mut buf = Vec::new();
        self.stream.read_to_end(&mut buf).ok()?;
        
        Some(String::from_utf8_lossy(&buf).trim().to_string())
    }
}

pub fn get_service(name: &str) -> Option<ServiceClient> {
    let path: PathBuf = ["/srv/", name].iter().collect();
    if !path.exists() {
        if name == "init" {
            return None;
        }

        let mut init = get_service("init")?;
        if init.call("launch_service", [name])? == "ERR" {
            return None;
        };
        std::thread::sleep(std::time::Duration::from_millis(100));

        return get_service(name);
    }
    
    let socket = match UnixStream::connect(path) {
        Ok(s) => s,
        Err(_) => return None,
    };

    ServiceClient::from_socket(socket)
}
