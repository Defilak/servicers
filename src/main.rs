use crate::child_proc::{ChildProcess,run_processes};
use crate::logger::log;
use std::env;

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
    use std::{sync::{Arc, Mutex, atomic::AtomicBool}, time::Duration, thread};

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

                let need_exit = Arc::new(AtomicBool::new(false));
                let threads = run_processes(list, &need_exit);
                
                while !threads.iter().all(|t| t.is_finished()) {
                    thread::sleep(Duration::from_millis(100));
                };
                Ok(())
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
        None => {

            let mut child_service1 = control::ServiceControl::new("GoodbyeDPI").unwrap();
            child_service1.stop()?;


            Ok(())
        },
    }
}
