use crate::child_proc::{run_processes, ChildProcess};
use crate::logger::log;
use std::env;
use std::{
    sync::{atomic::{AtomicBool, Ordering}, Arc},
    thread::{self, JoinHandle},
    time::Duration,
};

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

    use windows_service::service::ServiceState;

    use crate::control::ChildServiceControl;

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
                    Err(err) => log!("{:?}",&err),
                    _ => (),
                }
                Ok(())
            }
            _ => Ok(()),
        },
        None => {
            let mut threads = Vec::<JoinHandle<()>>::new();
            let need_exit = Arc::new(AtomicBool::new(false));
            let child_service1 = ChildServiceControl::new(proc_config::APACHE_SERVICE_NAME);
            if child_service1.is_ok() {
                let mut proc = child_service1.unwrap();
                let exit_flag = need_exit.clone();

                log!("Starting {}", proc.name);

                threads.push(thread::spawn(move || {
                    match proc.start() {
                        Ok(_) => log!("{} started", &proc.name),
                        Err(err) => log!("{:?}",&err),
                    };

                    loop {
                        if exit_flag.load(Ordering::Relaxed) == true {
                            log!("Stopping: {:?}", &proc.name);
                            match proc.stop() {
                                Ok(_) => log!("{} stopped", &proc.name),
                                Err(err) => log!("{:?}", &err),
                            };
                            break;
                        }

                        match proc.status() {
                            Ok(status) => {
                                if status.current_state != ServiceState::Running {
                                    log!("Restarting service {}: {:?}", &proc.name, status.current_state);
                                    match proc.start() {
                                        Ok(()) => log!("Service {} restarted",&proc.name),
                                        Err(err) => log!("Can't restart service {}: {}", &proc.name, err)
                                    };
                                }
                            },
                            Err(err) => log!("Can't get status for service {}: {}", &proc.name, err)
                        };

                        thread::sleep(Duration::from_millis(100));
                    }
                }));
            }
            while !threads.iter().all(|t| t.is_finished()) {
                thread::sleep(Duration::from_millis(100));
            }

            Ok(())
        }
    }
}
