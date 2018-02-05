use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::mem;
use std::os::unix::io::AsRawFd;
use std::thread;

#[allow(unused_imports)]
use libloading::{Library,Symbol};
use mio::{Events,Poll,PollOpt,Ready,Token};
use mio::unix::EventedFd;
use neli::socket::NlSocket;
use neli::ffi::NlFamily;

use acpi;
use evdev;
use netlink;

pub struct Eventer(Poll, NlSocket, Vec<File>);

impl Eventer {
    pub fn new() -> Result<Self, Box<Error>> {
        let netlink_id = netlink::resolve_acpi_family_id()?;
        let s = NlSocket::connect(NlFamily::Generic, None, Some(1 << (netlink_id - 1)))?;
        let event_files = evdev::evdev_files()?;
        Ok(Eventer(Poll::new()?, s, event_files))
    }

    pub fn setup_event_loop(&mut self) -> Result<(), Box<Error>> {
        self.0.register(&EventedFd(&self.1.as_raw_fd()), Token(0),
                        Ready::readable(), PollOpt::level())?;
        for (i, file) in self.2.iter().enumerate() {
            self.0.register(&EventedFd(&file.as_raw_fd()),
                            Token(i + 1), Ready::readable(), PollOpt::level())?;
        }
        Ok(())
    }

    pub fn start_event_loop(&mut self) -> Result<(), Box<Error>> {
        let mut events = Events::with_capacity(16);
        while let Ok(_) = self.0.poll(&mut events, None) {
            for event in events.iter() {
                if event.token() == Token(0) && event.readiness().is_readable() {
                    let acpi_event = netlink::acpi_listen(&mut self.1)?;
                    spawn_nl_acpi_thread(acpi_event)?;
                } else if event.readiness().is_readable() {
                    let index = event.token().0;
                    if let Some(mut f) = self.2.get(index) {
                        let mut buf = vec![0; mem::size_of::<evdev::InputEvent>()];
                        f.read_exact(&mut buf)?;
                    }
                }
            }
        }
        Ok(())
    }
}

fn spawn_nl_acpi_thread(event: acpi::AcpiEvent) -> Result<(), Box<Error>> {
    thread::spawn(move || {

    });
    Ok(())
}
