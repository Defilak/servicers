use std::env;
use crate::child_proc::ChildProcess;
use std::sync::atomic::AtomicBool;

mod tests;
mod child_proc;
mod control;
mod monitor_service;
mod proc_config;

pub const SERVICE_NAME: &str = "rtest";

#[cfg(not(windows))]
pub fn main() {
    panic!("This program is only intended to run on Windows.");
}

#[cfg(windows)]
fn main() -> windows_service::Result<()> {
    use std::sync::Arc;

    let args: Vec<String> = env::args().collect();

    match args.get(1) {
        Some(cmd) => match cmd.as_str() {
            "install" => control::install(),
            "uninstall" => control::uninstall(),
            "start" => control::start(),
            "stop" => control::stop(),
            "pause" => control::pause(),
            "resume" => control::resume(),
            "status" => control::status(),
            "run" => {
                let mut list = Vec::<ChildProcess>::new();
                for cfg in proc_config::load() {
                    list.push(ChildProcess::from_config(cfg));
                }

                let need_exit_flag = Arc::new(AtomicBool::new(false));
                monitor_service::run_processes(list, &need_exit_flag);
                loop {}
            }
            _ => Ok(()),
        },
        None => monitor_service::run(),
    }
}