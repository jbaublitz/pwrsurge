use std::collections::{self,HashMap};
use std::error::Error;
use std::fs::File;
use std::io::{self,Read};
use std::mem;
use std::os::unix::io::AsRawFd;
use std::slice;

use libc;
use mio::{self,Evented};
use mio::unix::EventedFd;
use tokio::prelude::{Async,Stream};

#[derive(Debug)]
struct EvdevEvents(HashMap<String, String>);

impl EvdevEvents {
    fn parse_file_chunk(&mut self, file_chunk: String) {
        let mut name = String::new();
        let mut handler_name = String::new();
        for line in file_chunk.lines() {
            let contains_name = line.contains("Name=");
            let contains_handlers = line.contains("Handlers=");
            if contains_name {
                let len = line.len();
                name = line[r#"N: Name=""#.len()..len - 1].to_string();
            } else if contains_handlers {
                let handlers = line["H: Handlers=".len()..].to_string();
                let handler_iter = handlers.split(" ");
                for handler in handler_iter {
                    if handler.contains("event") {
                        handler_name = handler.to_string();
                        break;
                    }
                }
            }
        }
        self.0.insert(handler_name, name);
    }

    fn parse_events_file(&mut self) -> Result<(), io::Error> {
        let file_name = "/proc/bus/input/devices";
        let file = File::open(file_name)?;
        let mut file_contents = String::new();
        file.take(65536).read_to_string(&mut file_contents)?;
        for file_split in file_contents.split("\n\n") {
            if file_split.trim() != "" {
                self.parse_file_chunk(file_split.to_string());
            }
        }
        Ok(())
    }

    pub fn parse_events() -> Result<Self, io::Error> {
        let mut evdev_events = EvdevEvents(HashMap::new());
        evdev_events.parse_events_file()?;
        Ok(evdev_events)
    }

    pub fn iter(&self) -> collections::hash_map::Iter<String, String> {
        self.0.iter()
    }
}

#[repr(C)]
pub struct InputEvent {
    pub timestamp: libc::timeval,
    pub event_type: u16,
    pub event_code: u16,
    pub event_value: i32,
}

impl Default for InputEvent {
    fn default() -> Self {
        InputEvent {
            timestamp: libc::timeval { tv_sec: 0, tv_usec: 0 },
            event_type: 0,
            event_code: 0,
            event_value: 0,
        }
    }
}

impl AsMut<[u8]> for InputEvent {
    fn as_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self as *mut InputEvent as *mut u8,
                                           mem::size_of::<InputEvent>()) }
    }
}

pub struct EventedFile(File);

impl EventedFile {
    fn open(path: &str) -> io::Result<Self> {
        Ok(EventedFile(File::open(path)?))
    }
}

impl Stream for EventedFile {
    type Item = InputEvent;
    type Error = Box<Error>;

    fn poll(&mut self) -> Result<Async<Option<Self::Item>>, Self::Error> {
        let mut input = InputEvent::default();
        let bytes_read = self.read(input.as_mut())?;
        if bytes_read == 0 {
            Ok(Async::NotReady)
        } else {
            Ok(Async::Ready(Some(input)))
        }
    }
}

impl Evented for EventedFile {
    fn register(&self, poll: &mio::Poll, token: mio::Token, ready: mio::Ready, opts: mio::PollOpt)
                -> io::Result<()> {
        poll.register(&EventedFd(&self.0.as_raw_fd()), token, ready, opts)
    }

    fn reregister(&self, poll: &mio::Poll, token: mio::Token, ready: mio::Ready, opts: mio::PollOpt)
                  -> io::Result<()> {
        poll.reregister(&EventedFd(&self.0.as_raw_fd()), token, ready, opts)
    }

    fn deregister(&self, poll: &mio::Poll) -> io::Result<()> {
        poll.deregister(&EventedFd(&self.0.as_raw_fd()))
    }
}

impl Read for EventedFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

pub fn evdev_files<'a>() -> Result<Vec<EventedFile>, Box<Error>> {
    let events = EvdevEvents::parse_events()?;
    let mut event_files = Vec::new();
    for (event, desc) in events.iter() {
        println!("Opening {} ({}) for reading...", event, desc);
        let file = EventedFile::open(format!("/dev/input/{}", event).as_str())?;
        event_files.push(file);
    }
    Ok(event_files)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_file_chunk_parsing() {
        let file_chunk = r#"I: Bus=0000 Vendor=0000 Product=0000 Version=0000
N: Name="HDA Intel PCH Mic"
P: Phys=ALSA
S: Sysfs=/devices/pci0000:00/0000:00:1f.3/sound/card0/input15
U: Uniq=
H: Handlers=event8
B: PROP=0
B: EV=21
B: SW=10"#;
        let mut evevents = EvdevEvents(HashMap::new());
        evevents.parse_file_chunk(file_chunk.to_string());
        assert_eq!(evevents.0.get(&"event8".to_string()), Some(&"HDA Intel PCH Mic".to_string()));
    }

    #[test]
    #[ignore]
    fn test_parse_event_file() {
        let mut evdev_events = EvdevEvents(HashMap::new());
        evdev_events.parse_events_file().unwrap();
        assert_eq!(evdev_events.0.get(&"event2".to_string()),
            Some(&"Lid Switch".to_string()));
    }
}
