use std::error::Error;
use std::sync::Arc;

use libloading::{Library,Symbol};
use neli::{Nl,MemWrite};
use neli::ffi::{NlFamily};
use neli::socket::NlSocket;
use tokio::{self,spawn};
use tokio::prelude::{future,Stream};

use acpi::AcpiEvent;
use netlink::acpi_event;
use evdev::{self,InputEvent,EventedFile};
use filter::{AcpiFilter,EvdevFilter};
use netlink;

pub fn new_event_loop(lib_path: &str, acpi_filter: AcpiFilter,
                      evdev_filter: EvdevFilter) -> Result<(), Box<Error>> {
    let netlink_id = netlink::resolve_acpi_family_id()?;
    let socket = NlSocket::connect(NlFamily::Generic, None, Some(1 << (netlink_id - 1)))?;
    let event_files = evdev::evdev_files()?;
    let lib = Arc::new(Library::new(lib_path)?);

    let socket_foreach = socket.for_each(move |item| {
        let acpi_event = match acpi_event(item) {
            Ok(ev) => ev,
            Err(_) => return spawn(future::empty()),
        };
        println!("device_class: {}", acpi_event.device_class);
        if acpi_filter.contains_device_class(&acpi_event.device_class) {
            let libref = Arc::clone(&lib);
            spawn(future::lazy(move || {
                type AcpiHandler<'sym> = Symbol<'sym, unsafe extern fn(*const AcpiEvent) -> i32>;
                let f = match unsafe { libref.get::<AcpiHandler>(b"acpi_handler") } {
                    Ok(f) => f,
                    Err(e) => {
                        println!("Failed to load acpi_handler from library: {}", e);
                        return Err(());
                    }
                };
                let buf = &mut [0; 43];
                let mut mwrite = MemWrite::new_slice(buf);
                match acpi_event.serialize(&mut mwrite) {
                    Ok(_) => (),
                    Err(_) => return Err(()),
                };
                unsafe { f(mwrite.as_slice() as *const _ as *const AcpiEvent) };
                Ok(())
            }))
        } else {
            spawn(future::empty())
        }
    });
    tokio::run(socket_foreach);
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
