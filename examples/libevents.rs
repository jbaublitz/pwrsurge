use std::fs::{self,File};
use std::io::{self,Read,Write};
use std::num;
use std::slice::from_raw_parts;
use std::str::{self,from_utf8};

#[macro_use]
mod macros;

struct ErrCode(i32);

impl Into<i32> for ErrCode {
    fn into(self) -> i32 {
        self.0
    }
}

from_error!(io::Error, str::Utf8Error, num::ParseIntError);

#[no_mangle]
pub extern "C" fn battery(bus_id: *const u8, _event_type: u32, _event_data: u32) -> i32 {
    let mut bus_id_bytes = unsafe { from_raw_parts(bus_id, 15) }.to_vec();
    bus_id_bytes = bus_id_bytes.into_iter().filter(|val| *val != 0).collect();
    let bus_id_string = return_errcode!(from_utf8(&bus_id_bytes));
    let dir = format!("/sys/bus/acpi/devices/{}/power_supply/", bus_id_string);
    let files = return_errcode!(fs::read_dir(dir.clone()));
    for file in files {
        let file_string = return_errcode!(file.and_then(|f| {
            let path = f.path();
            Ok(path.to_str().and_then(|s| Some(s.to_string())).unwrap_or(String::new()))
        }));
        let mut full_charge = return_errcode!(File::open(
            format!("{}/charge_full", file_string)
        ));
        let mut charge_now = return_errcode!(File::open(
            format!("{}/charge_now", file_string)
        ));
        let mut full_charge_string = String::new();
        return_errcode!(full_charge.read_to_string(&mut full_charge_string));
        let full_charge = return_errcode!(full_charge_string.trim().parse::<u64>());
        let mut charge_now_string = String::new();
        return_errcode!(charge_now.read_to_string(&mut charge_now_string));
        let charge_now = return_errcode!(charge_now_string.trim().parse::<u64>());

        let percent = (charge_now as f64 / full_charge as f64) * 100.0;
        let percent_int = percent as u64;

        if percent_int < 20 {
            let mut power_file = return_errcode!(File::create("/sys/power/state"));
            return_errcode!(power_file.write(b"disk"));
        }
    }
    0
}

pub fn main() {
}
