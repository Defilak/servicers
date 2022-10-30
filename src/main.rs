use std::env;
use std::{
    sync::{atomic::AtomicBool, Arc},
    thread,
    time::Duration,
};

use crate::child_proc::{run_processes, ChildProcess};
use crate::logger::log;

mod child_proc;
mod child_service;
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
    let args: Vec<String> = env::args().collect();

    match args.get(1) {
        Some(cmd) => match cmd.as_str() {
            "install" => control::install(),
            "uninstall" => control::uninstall(),
            "start" => control::start(),
            "stop" => control::stop(),
            "pause" => control::pause(),
            "resume" => control::resume(),
            "status" => {
                let stat = control::status();
                println!("{:?}", stat);
                stat
            }
            "run" => {
                let mut list = Vec::<ChildProcess>::new();
                for cfg in proc_config::load() {
                    list.push(ChildProcess::from_config(cfg));
                }

                let need_exit = Arc::new(AtomicBool::new(false));
                let threads = run_processes(list, &need_exit);

                while !threads.iter().all(|t| t.is_finished()) {
                    thread::sleep(Duration::from_millis(100));
                }
                Ok(())
            }
            "runservice" => {
                match monitor_service::run() {
                    Err(err) => log!("{:?}", &err),
                    _ => (),
                }
                Ok(())
            }
            _ => Ok(()),
        },
        None => {
            println!("Using: servicers.exe <command>");
            println!("Available commands: install, uninstall, start, stop, run, runservice");

            Ok(())
        }
    }
}
