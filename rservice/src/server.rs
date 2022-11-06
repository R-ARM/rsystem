use std::{
    sync::{
        Arc,
        Mutex,
    },
    thread,
    io::{Read, Write, BufReader, BufRead},
    path::PathBuf,
    collections::HashMap,
    os::unix::net::{UnixStream, UnixListener},
};

#[macro_export]
macro_rules! srv_fn {
    ($a:expr) => {
        (stringify!($a), $a as fn(Arc<Mutex<Box<_>>>, Vec<_>) -> String)
    }
}
pub use srv_fn;

//type FnMap: HashMap<String, fn(&Arc<Mutex<Box<T>>>, Vec<String>)>;
type Method<T> = fn(Arc<Mutex<Box<T>>>, Vec<String>) -> String;

pub struct ServiceServer<T> {
    state: Arc<Mutex<Box<T>>>,
    listener: UnixListener,
    methods: Arc<HashMap<String, Method<T>>>,
}

fn handle_client<T>(stream: UnixStream, methods: Arc<HashMap<String, Method<T>>>, state: Arc<Mutex<Box<T>>>) -> Option<()> {
    let mut argv_str = String::new();
    let mut reader = BufReader::new(stream);
    reader.read_line(&mut argv_str).ok()?;

    if argv_str.chars().filter(|v| v == &':').count() == 0 {
        argv_str.pop();
        argv_str.push(':');
        argv_str.push('\n');
    }

    let mut argv: Vec<String> = argv_str
        .split(':')
        .map(|v| v.to_string())
        .filter(|v| v.len() > 0)
        .collect();

    println!("{:#?}", &argv);
    let method = argv.remove(0);

    println!("{:#?}", &argv);
    let response = if let Some(function) = methods.get(&method) {
        function(state, argv)
    } else {
        String::from("\n")
    };

    let mut stream = reader.into_inner();
    stream.write_all(response.as_str().as_bytes()).ok()?;
    stream.shutdown(std::net::Shutdown::Both).ok()?;
    Some(())
}

impl<T: Send + Sync + 'static> ServiceServer<T> {
    pub fn new(name: &str, methods: impl IntoIterator<Item = (impl ToString, Method<T>)>, v: T) -> Option<ServiceServer<T>> {
        let path: PathBuf = ["/srv/", name].iter().collect();
        if path.exists() {
            return None;
        }

        let listener = UnixListener::bind(path).ok()?;

        Some(Self {
            state: Arc::new(Mutex::new(Box::new(v))),
            listener,
            methods: Arc::new(methods.into_iter().map(|(v1, v2)| (v1.to_string(), v2)).collect()),
        })
    }
    pub fn run(self) -> ! {
        loop {
            for stream in self.listener.incoming() {
                match stream {
                    Ok(s) => {
                        let state_ptr = Arc::clone(&self.state);
                        let method_ptr = Arc::clone(&self.methods);
                        thread::spawn(move || handle_client(s, method_ptr, state_ptr));
                    },
                    Err(e) => eprintln!("Failed to connect to client: {}", e),
                }
            }
        }
    }
}
