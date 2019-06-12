extern crate libc;

use std::error::Error;
use std::fs::{self,File,OpenOptions};
use std::io::{self,Read,Write};

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
pub struct AcpiEvent {
    pub device_class: String,
    pub bus_id: String,
    pub event_type: u32,
    pub event_data: u32,
}

#[repr(C)]
pub struct InputEvent {
    timestamp: libc::timeval,
    event_type: u16,
    event_code: u16,
    event_value: i32,
}

fn ac_is_online() -> Result<bool, io::Error> {
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

//fn parse_file_contents_to_percent(bat_dir_str: &String, bat_direntry: &String)
//        -> Result<u64, Box<dyn Error>> {
//    let bat_full_file_string = format!("{}/{}/charge_full", bat_dir_str,
//                                       bat_direntry);
//    let mut bat_full_file = File::open(bat_full_file_string.as_str())?;
//    let mut bat_full_string = String::new();
//    bat_full_file.read_to_string(&mut bat_full_string)?;
//    let bat_full_level = bat_full_string.trim().parse::<f64>()?;
//
//    let bat_now_file_string = format!("{}/{}/charge_now", bat_dir_str,
//                                       bat_direntry);
//    let mut bat_now_file = File::open(bat_now_file_string.as_str())?;
//    let mut bat_now_string = String::new();
//    bat_now_file.read_to_string(&mut bat_now_string)?;
//    let bat_now_level = bat_now_string.trim().parse::<f64>()?;
//
//    Ok(((bat_now_level / bat_full_level) * 100.0) as u64)
//}

//fn check_all_batteries(bat_dir_str: &String, online: bool) -> Result<bool, Box<dyn Error>> {
//    let mut all_bats_below = true;
//    let bat_dir = fs::read_dir(bat_dir_str.as_str())?;
//    for bat_entry in bat_dir {
//        let bat_direntry = bat_entry?;
//        let is_dir = bat_direntry.file_type()?.is_dir();
//        if is_dir {
//            let direntry_string = match bat_direntry.file_name().into_string() {
//                Ok(string) => string,
//                Err(_) => {
//                    return Err(Box::new(io::Error::from(io::ErrorKind::InvalidData)));
//                }
//            };
//            let percent = parse_file_contents_to_percent(&bat_dir_str,
//                                                         &direntry_string)?;
//            println!("BATTERY PERCENT: {}", percent);
//
//            if percent >= 20 || online {
//                all_bats_below = false;
//            }
//        }
//    }
//    Ok(all_bats_below)
//}

//fn suspend_and_lock() -> Result<(), Box<dyn Error>> {
//    let uid = env::var("SUDO_UID")?.parse::<u32>()?;
//    Command::new("scrot").arg("/tmp/ss.png").uid(uid).gid(uid).status().and_then(|_| {
//        Command::new("convert").args(&["/tmp/ss.png", "-blur", "0x5", "/tmp/ssb.png"])
//            .uid(uid).gid(uid).status()
//    }).and_then(|_| {
//        Command::new("i3lock").args(&["-i", "/tmp/ssb.png"])
//            .uid(uid).gid(uid).status()
//    }).and_then(|_| {
//        Command::new("rm").args(&["-f", "/tmp/ss.png", "/tmp/ssb.png"])
//            .uid(uid).gid(uid).status()
//    }).and_then(|_| {
//        Command::new("pm-suspend").status()
//    })?;
//    Ok(())
//}

//fn battery() -> i32 {
//    let online = try_int!(ac_is_online());
//
//    let mut all_bats_below = true;
//    let readdir = try_int!(fs::read_dir("/sys/bus/acpi/drivers/battery"));
//    for entry in readdir {
//        let direntry = try_int!(entry);
//        let is_symlink = try_int!(direntry.file_type()).is_symlink();
//        if is_symlink {
//            let direntry_string = match direntry.file_name().into_string() {
//                Ok(string) => string,
//                Err(_) => {
//                    println!("Failed to convert to String type");
//                    return -1;
//                }
//            };
//            let bat_dir_str = format!("/sys/bus/acpi/drivers/battery/{}/power_supply",
//                                      direntry_string);
//            all_bats_below = try_int!(check_all_batteries(&bat_dir_str, online));
//        }
//    }
//
//    println!("ALL BATTERIES BELOW THRESHOLD: {}", all_bats_below);
//    if all_bats_below {
//        try_int!(suspend_and_lock());
//    }
//
//    0
//}

fn battery(event: &AcpiEvent) -> i32 {
    println!("Device class: {}", event.device_class);
    println!("Bus ID: {}", event.bus_id);
    println!("Event type: {}", event.event_type);
    println!("Event data: {}", event.event_data);

    0
}

fn assert_cpu_state(is_online: bool, path: &str) -> Result<(), Box<dyn Error>> {
    let mut rw_file = OpenOptions::new().read(true).write(true).open(path)?;
    let mut state = String::new();
    rw_file.read_to_string(&mut state)?;
    let state_len = state.len();
    state.truncate(state_len - 1);
    let needs_state_change = ((state.as_str() != "powersave") && !is_online)
        || ((state.as_str() != "performance") && is_online);
    if needs_state_change {
        if is_online {
            rw_file.write(b"performance")?;
        } else {
            rw_file.write(b"powersave")?;
        }
    }
    Ok(())
}

fn assert_all_cpu_states(is_online: bool) -> Result<(), Box<dyn Error>> {
    let readdir = fs::read_dir("/sys/bus/cpu/devices/")?;
    for direntry in readdir {
        let cpu_device = direntry?;
        let is_symlink = cpu_device.file_type()?.is_symlink();
        if is_symlink {
            let direntry_string = match cpu_device.file_name().into_string() {
                Ok(string) => string,
                Err(_) => {
                    println!("Failed to convert to String type");
                    return Err(Box::new(io::Error::from(io::ErrorKind::InvalidData)));
                }
            };
            let cpu_path = format!("/sys/bus/cpu/devices/{}/cpufreq/scaling_governor",
                                   direntry_string);
            println!("Asserting CPU state for {}", cpu_path);
            assert_cpu_state(is_online, cpu_path.as_str())?;
        }
    }
    Ok(())
}

fn ac_adapter() -> i32 {
    let is_online = try_int!(ac_is_online());
    try_int!(assert_all_cpu_states(is_online));
    0
}

#[no_mangle]
pub fn acpi_handler(event: *const AcpiEvent) -> i32 {
    let event_ref = unsafe { &*event };

    match event_ref.device_class.as_str() {
        "battery" => battery(event_ref),
        "ac_adapter" => ac_adapter(),
        _ => 0,
    }
}

//#[no_mangle]
//pub fn evdev_handler(event: *const InputEvent) -> i32 {
//    let event_ref = unsafe { &*event };
//    if event_ref.event_type == 0x5 && event_ref.event_code == 0x0 && event_ref.event_value != 0 {
//        try_int!(suspend_and_lock());
//    }
//    0
//}

#[no_mangle]
pub fn evdev_handler(event: *const InputEvent) -> i32 {
    let event_ref = unsafe { &*event };
    println!("Event type: {}", event_ref.event_type);
    println!("Event code: {}", event_ref.event_code);
    println!("Event value: {}", event_ref.event_value);

    0
}

/// Only for `examples` directory to compile on `cargo test`
pub fn main() {
}
