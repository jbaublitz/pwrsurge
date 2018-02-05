use std::error::Error;
use std::fs::{self,File};
use std::io::{self,Read};
use std::process::Command;
use std::str::from_utf8;

macro_rules! try_int {
    ( $( $tokens:tt )* ) => {
        match $( $tokens )* {
            Ok(val) => val,
            Err(e) => {
                println!("{}", e.description());
                return -1;
            },
        }
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct AcpiEvent {
    pub device_class: [u8; 20],
    pub bus_id: [u8; 15],
    pub event_type: u32,
    pub event_data: u32,
}

#[derive(Debug)]
#[repr(C)]
pub struct ACState {
    is_unplugged: u8,
}

fn is_online() -> Result<bool, io::Error> {
    let readdir = fs::read_dir("/sys/bus/acpi/drivers/ac/")?;
    let mut online = false;
    for entry in readdir {
        let direntry = entry?;
        let is_symlink = direntry.file_type()?.is_symlink();
        if is_symlink {
            let direntry_string = match direntry.file_name().into_string() {
                Ok(string) => string,
                Err(_) => {
                    println!("Unsuccessful conversion from symbolic link name to string");
                    return Err(io::Error::from(io::ErrorKind::InvalidInput));
                },
            };
            let ac_file = format!("/sys/bus/acpi/drivers/ac/{}/power_supply/AC/online",
                                  direntry_string);
            let mut file = File::open(ac_file.as_str())?;
            let mut online_string = String::new();
            file.read_to_string(&mut online_string)?;
            let online_str = online_string.trim();
            if online_str == "1" {
                online = true;
            }
        }
    }
    Ok(online)
}

fn parse_file_contents_to_percent(bat_dir_str: &String, bat_direntry: &String)
        -> Result<u64, Box<Error>> {
    let bat_full_file_string = format!("{}/{}/charge_full", bat_dir_str,
                                       bat_direntry);
    let mut bat_full_file = File::open(bat_full_file_string.as_str())?;
    let mut bat_full_string = String::new();
    bat_full_file.read_to_string(&mut bat_full_string)?;
    let bat_full_level = bat_full_string.trim().parse::<f64>()?;

    let bat_now_file_string = format!("{}/{}/charge_now", bat_dir_str,
                                       bat_direntry);
    let mut bat_now_file = File::open(bat_now_file_string.as_str())?;
    let mut bat_now_string = String::new();
    bat_now_file.read_to_string(&mut bat_now_string)?;
    let bat_now_level = bat_now_string.trim().parse::<f64>()?;

    Ok(((bat_now_level / bat_full_level) * 100.0) as u64)
}

fn check_all_batteries(bat_dir_str: &String, online: bool) -> Result<bool, Box<Error>> {
    let mut all_bats_below = true;
    let bat_dir = fs::read_dir(bat_dir_str.as_str())?;
    for bat_entry in bat_dir {
        let bat_direntry = bat_entry?;
        let is_dir = bat_direntry.file_type()?.is_dir();
        if is_dir {
            let direntry_string = match bat_direntry.file_name().into_string() {
                Ok(string) => string,
                Err(_) => {
                    return Err(Box::new(io::Error::from(io::ErrorKind::InvalidData)));
                }
            };
            let percent = parse_file_contents_to_percent(&bat_dir_str,
                                                         &direntry_string)?;
            println!("BATTERY PERCENT: {}", percent);

            if percent >= 20 || online {
                all_bats_below = false;
            }
        }
    }
    Ok(all_bats_below)
}

fn battery() -> i32 {
    let online = try_int!(is_online());

    let mut all_bats_below = true;
    let readdir = try_int!(fs::read_dir("/sys/bus/acpi/drivers/battery"));
    for entry in readdir {
        let direntry = try_int!(entry);
        let is_symlink = try_int!(direntry.file_type()).is_symlink();
        if is_symlink {
            let direntry_string = match direntry.file_name().into_string() {
                Ok(string) => string,
                Err(_) => {
                    println!("Failed to convert to String type");
                    return -1;
                }
            };
            let bat_dir_str = format!("/sys/bus/acpi/drivers/battery/{}/power_supply",
                                      direntry_string);
            all_bats_below = try_int!(check_all_batteries(&bat_dir_str, online));
        }
    }

    println!("ALL BATTERIES BELOW THRESHOLD: {}", all_bats_below);
    if all_bats_below {
        try_int!(Command::new("sudo").arg("pm-suspend").spawn());
    }

    0
}

#[no_mangle]
pub fn handler(event: *const AcpiEvent) -> i32 {
    let event_ref = unsafe { &*event };
    let pos = event_ref.device_class.iter().position(|b| *b == 0);
    let device_class_clone = event_ref.device_class.clone();

    let device_class_str;
    if let Some(p) = pos {
        let (device_class_bytes, _) = device_class_clone.split_at(p);
        device_class_str = try_int!(from_utf8(device_class_bytes));
    } else {
        device_class_str = try_int!(from_utf8(&device_class_clone));
    }

    match device_class_str {
        "battery" => battery(),
        _ => 0,
    }
}

pub fn main() {
}
