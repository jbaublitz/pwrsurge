use std::env;
use std::error::Error;
use std::fmt;
use std::io;
use std::thread;

use libloading::Library;
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
    thread_local! {
        static LIB: Result<Library, io::Error> = Library::new(env::args().nth(1)
                                                              .unwrap_or("./test.so".to_string()));
    }
    while let Ok(event) = netlink::acpi_listen(s) {
        spawn_event_thread(event);
    }
    Ok(())
}

fn spawn_event_thread(event: AcpiEvent) {
    thread::spawn(move || {
        println!("{:?}", event);
    });
}
