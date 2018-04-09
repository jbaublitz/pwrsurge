use std::error::Error;
use std::fs::File;

use libloading::{Library,Symbol};
use mio::unix::EventedFd;
use neli::err::NlError;
use neli::ffi::{NlFamily,GenlId};
use neli::genlhdr::GenlHdr;
use neli::socket::NlSocket;
use tokio;
use tokio::prelude::{future,Async,Future,Stream};
use tokio::prelude::stream::ForEach;
use tokio::reactor::PollEvented2;
use tokio::executor::Spawn;

use acpi::AcpiEvent;
use netlink::acpi_event;
use evdev::{self,InputEvent,EventedFile};
use filter::{AcpiFilter,EvdevFilter};
use netlink;

pub fn new_event_loop(lib_path: &str, acpi_filter: AcpiFilter,
                      evdev_filter: EvdevFilter) -> Result<(), Box<Error>> {
    let lib = Library::new(lib_path)?;
    let netlink_id = netlink::resolve_acpi_family_id()?;
    let socket = NlSocket::connect(NlFamily::Generic, None, Some(1 << (netlink_id - 1)))?;
    let event_files = evdev::evdev_files()?;

    let socket = socket.for_each(move |item| {
        let acpi_event = match acpi_event(item) {
            Ok(ev) => ev,
            Err(e) => {
                println!("{}", e);
                return tokio::spawn(future::empty());
            }
        };
        if acpi_filter.contains_device_class(&acpi_event.device_class) {
            type AcpiHandler<'sym> = Symbol<'sym, unsafe extern fn(*const AcpiEvent) -> i32>;
            let f = unsafe { lib.get::<AcpiHandler>(b"acpi") }.unwrap();
            unsafe { f(&acpi_event as *const AcpiEvent) };
            tokio::spawn(future::empty())
        } else {
            tokio::spawn(future::empty())
        }
    });
    tokio::run(socket);
    println!("HERE");
    Ok(())
}

//fn spawn_nl_acpi_thread(lib: Arc<Library>, event: acpi::AcpiEvent) -> Result<(), Box<Error>> {
//    thread::spawn(move || {
//        type AcpiHandler<'sym> = Symbol<'sym, unsafe extern fn(*const acpi::AcpiEvent) -> i32>;
//        let func: Symbol<AcpiHandler> = match unsafe { lib.get::<AcpiHandler>(b"acpi_handler") } {
//            Ok(f) => f,
//            Err(e) => {
//                println!("Could not find acpi_handler function in library: {}", e);
//                return;
//            },
//        };
//        let mut mem = MemWrite::new_vec(Some(43));
//        event.serialize(&mut mem).unwrap();
//        let i = unsafe { func(mem.as_slice().as_ptr() as *const acpi::AcpiEvent) };
//        if i != 0 {
//            println!("acpi_handler exited unsuccessfully");
//        }
//    });
//    Ok(())
//}
//
//fn spawn_evdev_thread(lib: Arc<Library>, event: Vec<u8>) -> Result<(), Box<Error>> {
//    thread::spawn(move || {
//        type EvdevHandler<'sym> = Symbol<'sym, unsafe extern fn(*const evdev::InputEvent) -> i32>;
//        let func: Symbol<EvdevHandler> = match unsafe { lib.get::<EvdevHandler>(b"evdev_handler") } {
//            Ok(f) => f,
//            Err(e) => {
//                println!("Could not find evdev_handler function in library: {}", e);
//                return;
//            },
//        };
//        let i = unsafe { func(event.as_ptr() as *const evdev::InputEvent) };
//        if i != 0 {
//            println!("evdev_handler exited unsuccessfully");
//        }
//    });
//    Ok(())
//}
