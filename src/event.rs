use std::env;
use std::error::Error;
use std::fmt;
use std::io;
use std::str;
use std::thread;

use libloading::{self,Library,Symbol};
use neli::socket::NlSocket;

use acpi::AcpiEvent;
use netlink;

macro_rules! event_err {
    ( $err:ident, $( $from_err:path ),* ) => {
        #[derive(Debug)]
        pub struct $err(String);

        impl fmt::Display for $err {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", self.description())
            }
        }

        impl Error for $err {
            fn description(&self) -> &str {
                self.0.as_str()
            }
        }

        $(
            impl From<$from_err> for $err {
                fn from(v: $from_err) -> Self {
                    $err(v.description().to_string())
                }
            }
        )*
    }
}

event_err!(EventError, io::Error);

pub fn event_loop(s: &mut NlSocket) -> Result<(), EventError> {
    while let Ok(event) = netlink::acpi_listen(s) {
        spawn_event_thread(event);
    }
    Ok(())
}

fn spawn_event_thread<'a>(event: AcpiEvent) {
    thread_local! {
        static LIB: Result<Library, io::Error> = Library::new(env::args().nth(1)
                                                              .unwrap_or(
                                                                  "/etc/pwrsurge/libevents.so".to_string()
                                                              ));
    }
    thread::spawn(move || {
        type FuncSym<'sym> = Symbol<'sym, unsafe extern fn(*const u8, u32, u32) -> i32>;
        LIB.with(|lib| match *lib {
            Ok(ref l) => {
                let mut event_device_class = event.device_class.clone();
                event_device_class.retain(|elem| *elem != 0);
                let sym: libloading::Result<FuncSym> = unsafe { l.get(&event_device_class) };
                if let Ok(s) = sym {
                    let i = unsafe {
                        s((&event.bus_id).as_ptr(), event.event_type, event.event_data)
                    };
                    if i < 0 {
                        match str::from_utf8(&event_device_class) {
                            Ok(string) => {
                                println!("Thread for event {} exited unsuccessfully",
                                    string);
                            },
                            Err(e) => {
                                println!("Event name could not be turned into string: {}", e);
                            }
                        };
                    }
                } else if let Err(e) = sym {
                    println!("Failed to get symbol: {}", e);
                    println!("event: {:?}", event);
                }
            },
            Err(ref e) => { println!("Error loading library: {}", e); },
        });
    });
}
