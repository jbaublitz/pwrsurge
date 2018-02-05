use std::error::Error;
use std::env;

use getopts::Options;

pub struct PArgs {
    pub lib_path: String,
}

pub fn parse_args() -> Result<PArgs, Box<Error>> {
    let mut options = Options::new();
    options.optopt("l", "lib", "LIBRARY_PATH", "Path to plugin library");
    let matches = options.parse(env::args())?;

    let mut args = PArgs {
        lib_path: String::new(),
    };

    args.lib_path = matches.opt_str("l").unwrap_or("/etc/pwrsurge/libevents.so".to_string());
    Ok(args)
}
