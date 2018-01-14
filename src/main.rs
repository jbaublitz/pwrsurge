extern crate nl;

mod netlink;

use netlink::acpi_listen;

pub fn main() {
    acpi_listen().unwrap();
}
