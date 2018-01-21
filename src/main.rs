//! # Surge
//! ## An `nl`-based power manager
//!
//! ### Notes
//! This crate is currently meant as a reference implementation for how to use `nl`

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
