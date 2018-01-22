//! # pwrsurge
//! ## An `nl`-based power manager
//!
//! ### The power manager that allows you to drive the process
//! This power manager does very little heavy lifting. It simply subscribes to the ACPI
//! event family in netlink and calls out to the library that you specify from the command line
//! to execute callbacks.
//!
//! ### Hrm?
//! For examples of prototypes of callbacks in Rust and C, see the `example_libs/` directory. The
//! callbacks _must_ have the function prototypes in the examples specified for both languages
//! to have any guarantee of working. Otherwise, you are in uncharted waters and the behavior is
//! undefined. The `device_class` field of ACPI events should be the name of the function
//! which you wish to be executed on the event. Examples are `battery`, `cpu`, etc and running it
//! without these defined will print debugging information to the console so you can implement
//! them and know what events are happening which you might want to define behavior for.
//!
//! ### Usage
//! Running `pwrsurge PATH_TO_SHARED_LIBRARY` will allow you to specify the compiled libary object
//! containing the callbacks which you wish to be executed. If no arguments are specified, it will
//! default to `/etc/pwrsurge/libevents.so`. Please read the next section for security
//! considerations.
//!
//! ### Security - how is this okay?
//! This power manager might leave you thinking "how on earth is this okay?". That's a good
//! question. There are ways of using it that are decidedly *not* okay. One is running this as
//! root and specifying a library that is in a non-root user writable directory.
//! There is a potential race condition in which the user with write access can swap out the
//! library you've specified with something that should not have root access. If you are not
//! running this as root, this is somewhat less of a concern as the power manager will not execute
//! the code as root. This will resolve the privilege escalation concern.
//! However, best practice on single user systems is to make `/etc/pwrsurge/libevents.so`
//! world-readable and writable only by root.
//!
//! TD;DR *Never ever ever* run this program as root unless the library you are pointing to
//! and the directory that contains it is not writable by any user other than root. Even better,
//! make `/etc/pwrsurge/libevents.so` readable by all and writable only by root.

#![deny(missing_docs)]

extern crate neli;
extern crate libloading;

mod netlink;
mod acpi;
mod event;

use std::process;

use neli::socket::NlSocket;
use neli::ffi::NlFamily;

/// Main function
pub fn main() {
    let id = match netlink::resolve_acpi_family_id() {
        Ok(id) => id,
        Err(e) => {
            println!("{}", e);
            process::exit(1);
        }
    };
    let mut s = match NlSocket::connect(NlFamily::Generic, None, Some(1 << (id - 1))) {
        Ok(id) => id,
        Err(e) => {
            println!("{}", e);
            process::exit(1);
        }
    };
    match event::event_loop(&mut s) {
        Ok(id) => id,
        Err(e) => {
            println!("{}", e);
            process::exit(1);
        }
    };
}
