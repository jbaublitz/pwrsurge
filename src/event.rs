use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::mem;
use std::os::unix::io::AsRawFd;
use std::sync::Arc;
use std::thread;

use libloading::{Library,Symbol};
use mio::{Events,Poll,PollOpt,Ready,Token};
use mio::unix::EventedFd;
use neli::{Nl,NlSerState};
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

    pub fn start_event_loop(&mut self, lib_path: &str) -> Result<(), Box<Error>> {
        let lib = Arc::new(Library::new(lib_path)?);
        let mut events = Events::with_capacity(16);
        while let Ok(_) = self.0.poll(&mut events, None) {
            for event in events.iter() {
                if event.token() == Token(0) && event.readiness().is_readable() {
                    let acpi_event = netlink::acpi_listen(&mut self.1)?;
                    spawn_nl_acpi_thread(Arc::clone(&lib), acpi_event)?;
                } else if event.readiness().is_readable() {
                    let index = event.token().0 - 1;
                    if let Some(mut f) = self.2.get(index) {
                        let mut buf = vec![0; mem::size_of::<evdev::InputEvent>()];
                        f.read_exact(&mut buf)?;
                        spawn_evdev_thread(Arc::clone(&lib), buf)?;
                    }
                }
            }
        }
        Ok(())
    }
}

fn spawn_nl_acpi_thread(lib: Arc<Library>, mut event: acpi::AcpiEvent) -> Result<(), Box<Error>> {
    thread::spawn(move || {
        type AcpiHandler<'sym> = Symbol<'sym, unsafe extern fn(*const acpi::AcpiEvent) -> i32>;
        let func: Symbol<AcpiHandler> = match unsafe { lib.get::<AcpiHandler>(b"acpi_handler") } {
            Ok(f) => f,
            Err(e) => {
                println!("Could not find acpi_handler function in library: {}", e);
                return;
            },
        };
        let mut state = NlSerState::new();
        event.serialize(&mut state).unwrap();
        let i = unsafe { func(state.into_inner().as_ptr() as *const acpi::AcpiEvent) };
        if i != 0 {
            println!("acpi_handler exited unsuccessfully");
        }
    });
    Ok(())
}

fn spawn_evdev_thread(lib: Arc<Library>, event: Vec<u8>) -> Result<(), Box<Error>> {
    thread::spawn(move || {
        type EvdevHandler<'sym> = Symbol<'sym, unsafe extern fn(*const evdev::InputEvent) -> i32>;
        let func: Symbol<EvdevHandler> = match unsafe { lib.get::<EvdevHandler>(b"evdev_handler") } {
            Ok(f) => f,
            Err(e) => {
                println!("Could not find evdev_handler function in library: {}", e);
                return;
            },
        };
        let i = unsafe { func(event.as_ptr() as *const evdev::InputEvent) };
        if i != 0 {
            println!("evdev_handler exited unsuccessfully");
        }
    });
    Ok(())
}
