use std::env;

mod control;
mod monitor_service;

pub const SERVICE_NAME: &str = "rtest";

#[cfg(windows)]
fn main() -> windows_service::Result<()> {
    let args: Vec<String> = env::args().collect();

    match args.get(1) {
        Some(cmd) => match cmd.as_str() {
            "install" | "i" => control::install(),
            "uninstall" | "u" => control::uninstall(),
            "start" => control::start(),
            "stop" => control::stop(),
            "pause" => control::pause(),
            "resume" => control::resume(),
            "status" => control::status(),
            _ => Ok(()),
        },
        None => {
            monitor_service::run()
        },
    }
}

#[cfg(not(windows))]
pub fn main() {
    panic!("This program is only intended to run on Windows.");
}