//! # Surge
//! ## An `nl`-based power manager
//!
//! ### Notes
//! This crate is currently meant as a reference implementation for how to use `nl`

#![deny(missing_docs)]

extern crate nl;

mod netlink;
mod acpi;

use netlink::acpi_listen;

/// Main function
pub fn main() {
    acpi_listen().unwrap();
}
