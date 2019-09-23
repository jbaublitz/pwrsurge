use std::error::Error;
use std::sync::Arc;

use libloading::{Library, Symbol};
use neli::consts::{CtrlCmd, GenlId, NlFamily};
use neli::genl::Genlmsghdr;
use neli::socket::NlSocket;
use tokio::fs::File;
use tokio::prelude::{future, Future, Stream};
use tokio::{self, spawn};

use acpi::{acpi_event, AcpiEvent};
use evdev;
use filter::{AcpiFilter, EvdevFilter};

pub fn new_event_loop(
    lib_path: &str,
    acpi_filter: Arc<AcpiFilter>,
    evdev_filter: Arc<EvdevFilter>,
) -> Result<(), Box<dyn Error>> {
    let lib = Arc::new(Library::new(lib_path)?);
    let mut socket = NlSocket::connect(NlFamily::Generic, None, None, true)?;
    let id = socket.resolve_nl_mcast_group("acpi_event", "acpi_mc_group")?;
    socket.set_mcast_groups(vec![id])?;

    let lib_evdev = Arc::clone(&lib);
    let event_files = evdev::evdev_files()?.into_iter().map(move |string| {
        let evdev_filter_clone = Arc::clone(&evdev_filter);
        let lib_evdev_clone = Arc::clone(&lib_evdev);
        File::open(string)
            .map_err(|e| {
                println!("{}", e);
                ()
            })
            .and_then(move |f| {
                evdev::EvdevStream::new(f).for_each(move |item| {
                    let lib_evdev_foreach = Arc::clone(&lib_evdev_clone);
                    if !evdev_filter_clone.contains_code(&item.get_event_code())
                        || !evdev_filter_clone.contains_type(&item.get_event_type())
                        || !evdev_filter_clone.contains_value(&item.get_event_value())
                            && !evdev_filter_clone.is_wildcard()
                    {
                        return Ok(());
                    }
                    let _ = spawn(future::lazy(move || {
                        type InputHandler<'sym> =
                            Symbol<'sym, unsafe extern "C" fn(*const evdev::InputEvent) -> i32>;
                        let f = match unsafe {
                            lib_evdev_foreach.get::<InputHandler>(b"evdev_handler")
                        } {
                            Ok(f) => f,
                            Err(e) => {
                                println!("Failed to load acpi_handler from library: {}", e);
                                return Err(());
                            }
                        };
                        unsafe { f(item.as_buffer() as *const _ as *const evdev::InputEvent) };
                        Ok(())
                    }));
                    Ok(())
                })
            })
    });

    let lib_clone = Arc::clone(&lib);
    tokio::run(future::lazy(move || {
        for foreach in event_files {
            spawn(foreach);
        }
        let stream_socket = match neli::socket::tokio::NlSocket::new(socket) {
            Ok(ss) => ss,
            Err(e) => {
                println!("{}", e);
                return Err(());
            }
        };
        spawn(create_socket_event_loop(
            stream_socket,
            lib_clone,
            acpi_filter,
        ));
        Ok(())
    }));

    Ok(())
}

fn create_socket_event_loop(
    socket: neli::socket::tokio::NlSocket<GenlId, Genlmsghdr<CtrlCmd, u16>>,
    lib: Arc<Library>,
    acpi_filter: Arc<AcpiFilter>,
) -> impl Future<Item = (), Error = ()> {
    socket
        .for_each(move |item| {
            let acpi_event = match acpi_event(item) {
                Ok(ev) => ev,
                Err(_) => return Ok(()),
            };
            if !acpi_filter.contains_device_class(&acpi_event.device_class)
                && !acpi_filter.is_wildcard()
            {
                return Ok(());
            }
            let libref = Arc::clone(&lib);
            let _ = spawn(future::lazy(move || {
                type AcpiHandler<'sym> =
                    Symbol<'sym, unsafe extern "C" fn(*const AcpiEvent) -> i32>;
                let f = match unsafe { libref.get::<AcpiHandler>(b"acpi_handler") } {
                    Ok(f) => f,
                    Err(e) => {
                        println!("Failed to load acpi_handler from library: {}", e);
                        return Err(());
                    }
                };
                unsafe { f(&acpi_event as *const _ as *const AcpiEvent) };
                Ok(())
            }));
            Ok(())
        })
        .map_err(|e| {
            println!("{}", e);
            ()
        })
}
