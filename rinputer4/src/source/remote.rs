use std::net::{ToSocketAddrs, TcpStream};
use std::{
    io::{BufRead, BufReader, Read},
    sync::mpsc::{self, Sender, Receiver},
};
use evdev::{
    EventType,
    Key,
    InputEvent,
    AbsoluteAxisType,
};
use crate::{
    source::{
        EventSource,
        SourceCaps,
    },
};

pub struct Remote {
    stream: TcpStream,
    tx: Sender<InputEvent>,
    rx: Option<Receiver<InputEvent>>
}

unsafe impl Send for Remote {}
unsafe impl Sync for Remote {}

impl Remote {
    pub fn new(addr: impl ToSocketAddrs) -> Option<Self> {
        let stream = TcpStream::connect(addr).ok()?;
        let (tx, rx) = mpsc::channel();
        Some(Self {
            stream,
            tx,
            rx: Some(rx),
        })
    }
}

fn event_key(code: Key, value: &str) -> InputEvent {
    InputEvent::new(EventType::KEY, code.0, value.parse().unwrap())
}

fn event_abs(code: AbsoluteAxisType, value: i32) -> InputEvent {
    InputEvent::new(EventType::ABSOLUTE, code.0, value)
}

fn worker(mut dev: Remote) {
    let mut reader = BufReader::new(&mut dev.stream);
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).expect("Failed to read a line");

        let line_filtered = line
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>();
        let event_str = line_filtered.split(' ').collect::<Vec<&str>>();

        match event_str[0] {
            "BTN_WEST"          => dev.tx.send(event_key(Key::BTN_WEST, event_str[1])).unwrap(),
            "BTN_SOUTH"         => dev.tx.send(event_key(Key::BTN_SOUTH, event_str[1])).unwrap(),
            "BTN_SELECT"        => dev.tx.send(event_key(Key::BTN_SELECT, event_str[1])).unwrap(),
            "BTN_START"         => dev.tx.send(event_key(Key::BTN_START, event_str[1])).unwrap(),
            "BTN_DPAD_RIGHT"    => dev.tx.send(event_abs(AbsoluteAxisType::ABS_HAT0X, event_str[1].parse::<i32>().unwrap() )).unwrap(),
            "BTN_DPAD_LEFT"     => dev.tx.send(event_abs(AbsoluteAxisType::ABS_HAT0X, event_str[1].parse::<i32>().unwrap() * -1)).unwrap(),
            "BTN_DPAD_DOWN"     => dev.tx.send(event_abs(AbsoluteAxisType::ABS_HAT0Y, event_str[1].parse::<i32>().unwrap() )).unwrap(),
            "BTN_DPAD_UP"       => dev.tx.send(event_abs(AbsoluteAxisType::ABS_HAT0Y, event_str[1].parse::<i32>().unwrap() * -1)).unwrap(),
            "BTN_TR"            => dev.tx.send(event_key(Key::BTN_TR, event_str[1])).unwrap(),
            "BTN_TL"            => dev.tx.send(event_key(Key::BTN_TL, event_str[1])).unwrap(),
            "BTN_NORTH"         => dev.tx.send(event_key(Key::BTN_NORTH, event_str[1])).unwrap(),
            "BTN_WEST"          => dev.tx.send(event_key(Key::BTN_WEST, event_str[1])).unwrap(),
            "BTN_TL2"           => dev.tx.send(event_key(Key::BTN_WEST, event_str[1])).unwrap(),
            "BTN_TR2"           => dev.tx.send(event_key(Key::BTN_WEST, event_str[1])).unwrap(),
            _ => (),
        }
    }
}

impl EventSource for Remote {
    fn make_tx(&self) -> Sender<InputEvent> {
        self.tx.clone()
    }
    fn start_ev(mut self: Box<Remote>) -> Receiver<InputEvent> {
        let rx = self.rx.take();
        std::thread::spawn(|| worker(*self));
        rx.unwrap()
    }
    fn name(&self) -> String {
        String::from("Remote input device")
    }
    fn path(&self) -> String {
        "".to_string()
    }
    fn get_capabilities(&self) -> SourceCaps {
        SourceCaps::FullX360
    }
}
