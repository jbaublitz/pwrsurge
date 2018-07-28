use std::env;
use std::error::Error;
use std::process;

use getopts::Options;
use ini::Ini;

use filter::{AcpiFilter,EvdevFilter};

pub struct CfgFile {
    pub acpi: AcpiFilter,
    pub evdev: EvdevFilter,
    pub input: bool,
}

pub struct PArgs {
    pub lib_path: Box<str>,
    pub config_file: CfgFile,
}

pub fn parse_args() -> Result<PArgs, Box<Error>> {
    let mut options = Options::new();
    options.optopt("l", "lib", "LIBRARY_PATH", "Path to plugin library")
        .optopt("c", "config", "CONFIG_PATH", "Path to config file")
        .optflag("h", "help", "Help text");
    let matches = options.parse(env::args())?;

    if matches.opt_present("h") {
        println!("{}", options.usage("USAGE: pwrsurge [OPTIONS]"));
        process::exit(0);
    }

    let c_optstr = matches.opt_str("c");
    let config_path = c_optstr.as_ref().map(|s| s.as_ref());
    let cfg = parse_config(config_path.unwrap_or("/etc/pwrsurge/pwrsurge.conf"))?;

    let mut args = PArgs {
        lib_path: Box::from(""),
        config_file: cfg,
    };

    args.lib_path = matches.opt_str("l").map(|s| Box::from(s))
        .unwrap_or(Box::from("/usr/lib/pwrsurge/libevents.so"));
    Ok(args)
}

pub fn parse_acpi_config(ini: &Ini) -> AcpiFilter {
    let vec = match ini.section(Some("acpi")) {
        Some(acpi) => {
            let whitelist = acpi.get("device_class_whitelist").map(|s| s.to_owned());
            match whitelist {
                Some(wl) => wl.split(",").filter_map(|s| {
                        if s == "" { None } else { Some(s.to_string()) }
                    }).collect::<Vec<String>>(),
                _ => Vec::new(),
            }
        },
        _ => Vec::new(),
    };
    AcpiFilter::new(vec)
}

pub fn parse_evdev_config(ini: &Ini) -> EvdevFilter {
    match ini.section(Some("evdev")) {
        Some(evdev) => {
            let type_whitelist = evdev.get("type_whitelist").map(|s| s.to_owned())
                .unwrap_or(String::new()).split(",").filter_map(|s| s.parse::<u16>().ok())
                .collect::<Vec<u16>>();
            let code_whitelist = evdev.get("code_whitelist").map(|s| s.to_owned())
                .unwrap_or(String::new()).split(",").filter_map(|s| s.parse::<u16>().ok())
                .collect::<Vec<u16>>();
            EvdevFilter::new(type_whitelist, code_whitelist)
        },
        _ => EvdevFilter::new(Vec::new(), Vec::new()),
    }
}

pub fn parse_timer_config(ini: &Ini) -> bool {
    match ini.section(Some("timer")) {
        Some(timer) => {
            let input = timer.get("reset_on_input")
                .map(|v| v.parse::<bool>().unwrap_or(false)).unwrap_or(false);
            input
        },
        _ => false,
    }
}

pub fn parse_config(config_path: &str) -> Result<CfgFile, Box<Error>> {
    let ini = Ini::load_from_file(config_path)?;
    let acpi_section = parse_acpi_config(&ini);
    let evdev_section = parse_evdev_config(&ini);
    let input = parse_timer_config(&ini);
    Ok(CfgFile {
        acpi: acpi_section,
        evdev: evdev_section,
        input,
    })
}
