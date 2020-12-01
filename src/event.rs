use std::{error::Error, sync::Arc};

use futures_util::{future::select_all, select, FutureExt};
use libloading::{Library, Symbol};
use neli::{
    consts::{socket::*},
    socket::{NlSocketHandle, tokio::NlSocket},
    utils::{serialize, U32Bitmask, U32BitFlag},
};
use tokio::{
    fs::File,
    runtime::Runtime,
    stream::StreamExt,
    spawn,
};

use crate::{
    acpi::{acpi_event, AcpiEvent},
    evdev::{evdev_files, EvdevStream, InputEvent},
    filter::{AcpiFilter, EvdevFilter},
};

async fn handle_event(handler: Arc<Library>, item: InputEvent) {
    type InputHandler<'sym> =
        Symbol<'sym, unsafe extern "C" fn(*const InputEvent) -> i32>;
    let f = match unsafe { handler.get::<InputHandler>(b"evdev_handler") } {
        Ok(f) => f,
        Err(e) => {
            println!("Failed to load evdev_handler from library: {}", e);
            return;
        },
    };
    unsafe { f(item.as_buffer() as *const _ as *const InputEvent) };
}

async fn event_files(handler: Arc<Library>, evdev_filter: Arc<EvdevFilter>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut join_handles = vec![];
    for evdev_file in evdev_files()? {
        let file = File::open(evdev_file).await?;
        let handler_clone = Arc::clone(&handler);
        let evdev_filter_clone = Arc::clone(&evdev_filter);
        join_handles.push(spawn(async move {
            let mut evdev_stream = EvdevStream::new(file);
            loop {
                match evdev_stream.next().await {
                    Some(Ok(event)) => {
                        if evdev_filter_clone.contains_code(&event.get_event_code())
                            && evdev_filter_clone.contains_type(&event.get_event_type())
                            && evdev_filter_clone.contains_value(&event.get_event_value())
                            || evdev_filter_clone.is_wildcard()
                        {
                            let _ = spawn(handle_event(Arc::clone(&handler_clone), event));
                        }
                    },
                    Some(Err(e)) => return Err(e),
                    None => return Ok(()),
                }
            }
        }));
    }
    let _ = select_all(join_handles).await;
    Ok(())
}

async fn handle_acpi_event(lib: Arc<Library>, acpi_event: AcpiEvent) {
    type AcpiHandler<'sym> =
        Symbol<'sym, unsafe extern "C" fn(*const AcpiEvent) -> i32>;
    let f = match unsafe { lib.get::<AcpiHandler>(b"acpi_handler") } {
        Ok(f) => f,
        Err(e) => {
            println!("Failed to load acpi_handler from library: {}", e);
            return;
        },
    };
    let acpi_event_buffer = match serialize(&acpi_event, false) {
        Ok(buf) => buf,
        Err(e) => {
            println!("{}", e);
            return;
        }
    };
    unsafe { f(acpi_event_buffer.as_slice() as *const _ as *const AcpiEvent) };
}

async fn create_socket_event_loop(
    lib: Arc<Library>,
    acpi_filter: Arc<AcpiFilter>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut socket = NlSocketHandle::connect(NlFamily::Generic, None, U32Bitmask::empty())?;
    let id = socket.resolve_nl_mcast_group("acpi_event", "acpi_mc_group")?;
    socket.add_mcast_membership(U32Bitmask::from(U32BitFlag::new(id)?))?;

    let mut socket = NlSocket::new(socket)?;
    loop {
        match socket.next().await {
            Some(Ok(event)) => {
                let acpi_event = match acpi_event(event) {
                    Ok(ev) => ev,
                    Err(e) => return Err(Box::new(e)),
                };
                println!("{:?}", acpi_event);
                if acpi_filter.contains_device_class(&acpi_event.device_class.0)
                    || acpi_filter.is_wildcard() {
                    let _ = spawn(handle_acpi_event(Arc::clone(&lib), acpi_event));
                }
            },
            Some(Err(e)) => return Err(Box::new(e)),
            None => return Ok(()),
        }
    }
}

pub fn new_event_loop(
    lib_path: &str,
    acpi_filter: Arc<AcpiFilter>,
    evdev_filter: Arc<EvdevFilter>,
) -> Result<(), Box<dyn Error>> {
    let lib = Arc::new(Library::new(lib_path)?);
    let runtime = Runtime::new()?;
    runtime.block_on(async move {
        let lib_clone = Arc::clone(&lib);
        let evdev_handle = spawn(async move {
            if let Err(e) = event_files(lib_clone, evdev_filter).await {
                println!("{}", e);
            }
        });
        let netlink_handle = spawn(async move {
            if let Err(e) = create_socket_event_loop(
                lib,
                acpi_filter,
            ).await {
                println!("{}", e);
            }
        });
        select! {
            _ = evdev_handle.fuse() => {
                println!("evdev handler exited");
            }
            _ = netlink_handle.fuse() => {
                println!("netlink handler exited");
            }
        };
    });

    Ok(())
}
