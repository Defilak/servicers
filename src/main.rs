use crate::child_proc::ChildProcess;
use crate::logger::log;
use std::env;
use std::sync::atomic::AtomicBool;

mod child_proc;
mod control;
mod logger;
mod monitor_service;
mod proc_config;
mod tests;

pub const SERVICE_NAME: &str = "servicers";

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
            "runservice" => {
                match monitor_service::run() {
                    Err(err) => log(&err),
                    _ => ()
                }
                Ok(())
            }
            _ => Ok(()),
        },
        None => Ok(()),
    }
}
