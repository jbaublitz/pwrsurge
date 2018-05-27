use std::error::Error;
use std::sync::Arc;

use libloading::{Library,Symbol};
use neli::{Nl,MemWrite};
use neli::ffi::{CtrlCmd,GenlId,NlFamily};
use neli::genlhdr::GenlHdr;
use neli::socket::NlSocket;
use tokio::{self,spawn};
use tokio::fs::File;
use tokio::prelude::{future,Future,Stream};

use acpi::AcpiEvent;
use netlink::acpi_event;
use evdev;
use filter::{AcpiFilter,EvdevFilter};
use netlink;

pub fn new_event_loop(lib_path: &str, acpi_filter: Arc<AcpiFilter>,
                      evdev_filter: Arc<EvdevFilter>) -> Result<(), Box<Error>> {
    let lib = Arc::new(Library::new(lib_path)?);
    let netlink_id = netlink::resolve_acpi_family_id()?;
    let socket = NlSocket::connect(NlFamily::Generic, None, Some(1 << (netlink_id - 1)))?;

    let lib_evdev = Arc::clone(&lib);
    let event_files = evdev::evdev_files()?.into_iter().map(move |string| {
        let evdev_filter_clone = Arc::clone(&evdev_filter);
        let lib_evdev_clone = Arc::clone(&lib_evdev);
        File::open(string).map_err(|e| {
            println!("{}", e);
            ()
        }).and_then(move |f| evdev::EvdevStream::new(f).for_each(move |item| {
            let lib_evdev_foreach = Arc::clone(&lib_evdev_clone);
            if !evdev_filter_clone.contains_code(&item.event_code) &&
                    !evdev_filter_clone.contains_type(&item.event_type) &&
                    !evdev_filter_clone.is_wildcard() {
                return Ok(());
            }
            let _ = spawn(future::lazy(move || {
                type InputHandler<'sym> =
                    Symbol<'sym, unsafe extern fn(*const evdev::InputEvent) -> i32>;
                let f = match unsafe { lib_evdev_foreach.get::<InputHandler>(b"evdev_handler") } {
                    Ok(f) => f,
                    Err(e) => {
                        println!("Failed to load acpi_handler from library: {}", e);
                        return Err(());
                    }
                };
                unsafe { f(item.as_ref() as *const _ as *const evdev::InputEvent) };
                Ok(())
            }));
            Ok(())
        }))
    });

    let lib_clone = Arc::clone(&lib);
    tokio::run(future::lazy(move || {
        for foreach in event_files {
            spawn(foreach);
        }
        spawn(create_socket_event_loop(socket, lib_clone, acpi_filter));
        Ok(())
    }));

    Ok(())
}

fn create_socket_event_loop(socket: NlSocket<GenlId, GenlHdr<CtrlCmd>>, lib: Arc<Library>,
                            acpi_filter: Arc<AcpiFilter>) -> impl Future<Item = (), Error = ()> {
    socket.for_each(move |item| {
        let acpi_event = match acpi_event(item) {
            Ok(ev) => ev,
            Err(_) => return Ok(()),
        };
        if !acpi_filter.contains_device_class(&acpi_event.device_class) && !acpi_filter.is_wildcard() {
            return Ok(())
        }
        let libref = Arc::clone(&lib);
        let _ = spawn(future::lazy(move || {
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
        }));
        Ok(())
    })
}
