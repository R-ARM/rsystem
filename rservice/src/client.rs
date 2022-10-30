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
    methods: HashSet<String>,
    stream: UnixStream,
}

impl ServiceClient {
    fn from_socket(mut input: UnixStream) -> Option<Self> {
        let mut methods_string = String::new();
        input.read_to_string(&mut methods_string).ok()?;
        
        Some(Self {
            stream: input,
            methods: methods_string.split(':').map(|v| v.to_string()).collect(),
        })
    }
    pub fn call(&mut self, method: impl ToString, args: impl IntoIterator<Item = impl ToString>) -> Option<String> {
        if !self.methods.contains(&method.to_string()) {
            return None;
        }

        let mut tmp: Vec<String> = Vec::new();
        tmp.push(method.to_string());
        tmp.extend(args.into_iter().map(|v| v.to_string()));
        let mut full_call: String = tmp.into_iter()
            .map(|v| format!("{}:", v))
            .collect();
        full_call.pop();

        self.stream.write_all(full_call.as_str().as_bytes()).ok()?;

        let mut ret = String::new();
        self.stream.read_to_string(&mut ret).ok()?;
        
        Some(ret)
    }
}

pub fn get_service(name: &str) -> Option<ServiceClient> {
    let path: PathBuf = ["/srv/", name].iter().collect();
    if !path.exists() {
        return None;
    }
    
    let socket = match UnixStream::connect(path) {
        Ok(s) => s,
        Err(_) => return None,
    };

    ServiceClient::from_socket(socket)
}
