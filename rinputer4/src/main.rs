mod source;
mod sink;

use std::net::{
    TcpListener,
    TcpStream,
};
use std::io::{
    self,
    BufReader,
    BufRead,
    Write,
};
use std::sync::{
    Arc,
    Mutex,
};
use anyhow::Result;

use crate::{
    sink::{Sink, uinput::UinputSink},
    source::remote::Remote,
    source::OpenedEventSource,
};

static HELP_TEXT: &[u8] = b"Available commands are:
list_sinks: Lists all sinks in use with sources attached to them
add_sink: Adds a sink and autobinds a source
del_sink: Removes a sink
list_sink_types: Lists sink types that can be added with add_sink
help: Displays this message
";

fn handle_client(mut stream: TcpStream, all_sinks_mutex: Arc<Mutex<Vec<Box<dyn Sink>>>>) -> Result<()> {
    let sink_types = sink::list_names();

    loop {
        let mut buf = String::new();
        let mut buf_reader = BufReader::new(&mut stream);
        buf_reader.read_line(&mut buf)?;

        let args: Vec<&str> = buf.trim().split(' ').collect();

        match args[0] {
            "add_3ds" => {
                let mut all_sinks = all_sinks_mutex.lock().unwrap();
                let new = Remote::new("192.168.88.120:2137").unwrap();
                all_sinks.push(UinputSink::new(source::into_opened(Box::new(new)))?);
                stream.write_all(b"OK\n");
            },
            "add_sink" => {
                let snk_type = args[1].parse::<usize>()?;
                let mut all_sinks = all_sinks_mutex.lock().unwrap();
                let (_, new_fn) = sink_types[snk_type];

                let cur_sources = source::enumerate().into_iter()
                    .map(|v| source::into_opened(v))
                    .collect::<Vec<OpenedEventSource>>();
                let new_source = source::wait_for_lr(cur_sources);
                match new_fn(new_source) {
                    Ok(sink) => {
                        all_sinks.push(sink);
                        stream.write_all(b"OK\n")?;
                    }
                    Err(e) => {
                        eprintln!("Failed making a new sink:");
                        eprintln!("{}", e);
                        stream.write_all(b"ERR\n")?;
                    }
                };
            },
            "del_sink" => {
                let mut all_sinks = all_sinks_mutex.lock().unwrap();
                let victim = args[1].parse::<usize>()?;
                all_sinks.remove(victim);
                stream.write_all(b"OK\n")?;
            }
            "list_sink_types" => {
                for (i, (name, _)) in sink_types.iter().enumerate() {
                    let tmp = format!("OK:{}:{}", i, name);
                    stream.write_all(tmp.as_str().as_bytes())?;
                    stream.write_all(b"\n")?;
                }
                stream.write_all(b"END_MULTILINE\n")?;
            },
            "list_sinks" => {
                let all_sinks = all_sinks_mutex.lock().unwrap();
                for (i, sink) in all_sinks.iter().enumerate() {
                    let response = format!("OK:{}:{}:{}\n", i, sink.name(), sink.source_name());
                    stream.write_all(response.as_str().as_bytes())?;
                }
                stream.write_all(b"END_MULTILINE\n")?;
            }
            "help" => stream.write_all(HELP_TEXT)?,
            _ => stream.write_all(b"ERR:Invalid command\n")?,
        }

        println!("{:#?}", args);
    }
}

fn main() -> io::Result<()> {


    let mutex = Arc::new(Mutex::new(Vec::new()));
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    for stream in listener.incoming() {
        let ptr = Arc::clone(&mutex);
        std::thread::spawn(move || handle_client(stream.unwrap(), ptr));
    };

    Ok(())
    /*
    let devices = source::enumerate();
    for (i, device) in devices.iter().enumerate() {
        println!("Device {}: {}", i, device.name())
    }
    println!("Which device should be used for 1st gamepad?");

    let mut buf = String::new();
    io::stdin().read_line(&mut buf).unwrap();

    let dev_idx = buf.trim().parse::<usize>().unwrap();

    println!("Using device {}", devices[dev_idx].name());*/
}
