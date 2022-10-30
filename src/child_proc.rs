use crate::logger::log;
use crate::proc_config::*;
use std::process::{Child, Command};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

pub struct ChildProcess {
    pub config: ProcessConfig,
    child: Option<Child>,
}

impl ChildProcess {
    pub fn _new(program: &str, args: Vec<String>, workdir: String) -> ChildProcess {
        ChildProcess {
            config: ProcessConfig {
                program: program.to_string(),
                args: args,
                cwd: workdir,
                state: ProcessConfigState::Enabled,
            },
            child: None,
        }
    }

    pub fn from_config(config: ProcessConfig) -> ChildProcess {
        ChildProcess {
            config: config,
            child: None,
        }
    }

    pub fn run(&mut self, exit_flag: &Arc<Mutex<bool>>) {
        if self.config.is_valid() {
            println!("spawnthread");
            self.start();
        }

        loop {
            if exit_flag.lock().unwrap().eq(&true) {
                self.kill();
                break;
            }

            if self.config.is_valid() {
                self.try_restart();
            }

            thread::sleep(Duration::from_millis(100));
        }
    }

    pub fn start(&mut self) {
        self.child = match self.config.spawn_new() {
            Ok(child) => {
                Some(child)
            }
            Err(err) => {
                log!("{:?}", &err);
                None
            }
        };
    }

    pub fn start_restart_loop(&mut self) {
        self.start();

        loop {
            match self.child.as_mut().unwrap().wait() {
                Ok(e) => {
                    log!("{:?}", &e);
                }
                Err(e) => {
                    log!("{:?}", &e);
                }
            }
            self.start();
        }
    }

    pub fn try_restart(&mut self) -> bool {
        match self.child.as_mut().unwrap().try_wait() {
            Ok(Some(status)) => {
                dbg!(status);
                self.start();
                true
            }
            Err(e) => {
                dbg!(e);
                self.start();
                true
            }
            Ok(None) => false,
        }
    }

    pub fn kill(&mut self) {
        if let Some(child) = self.child.as_mut() {
            match child.kill() {
                Ok(e) => {
                    dbg!(e);
                }
                Err(err) => {
                    dbg!(err);
                }
            }
        }
    }
}

pub fn run_processes(list: Vec<ChildProcess>, exit_flag: &Arc<AtomicBool>) -> Vec<JoinHandle<()>> {
    let mut threads = Vec::<JoinHandle<()>>::new();
    for mut proc in list {
        // Для каждого копирую ссылку
        let exit_flag = exit_flag.clone();

        threads.push(thread::spawn(move || {
            if !proc.config.is_valid() {
                log!("Invalid config: {:?}", &proc.config);
                return;
            }

            log!("Starting: {:?}", &proc.config);
            proc.start();

            loop {
                if exit_flag.load(Ordering::Relaxed) == true {
                    log!("Killing: {:?}", &proc.config);
                    proc.kill();
                    break;
                }

                if proc.config.is_valid() {
                    if proc.try_restart() {
                        log!("Restarting: {:?}", &proc.config);
                    }
                }

                thread::sleep(Duration::from_millis(100));
            }
        }));
    }

    threads
}

#[test]
fn test_run() {
    let mut list = Vec::<ChildProcess>::new();
    for cfg in super::proc_config::load() {
        list.push(ChildProcess::from_config(cfg));
    }

    let need_exit = Arc::new(AtomicBool::new(false));
    let threads = run_processes(list, &need_exit);

    thread::sleep(Duration::from_secs(5));
    need_exit.store(true, Ordering::Relaxed);

    Command::new(&NGINX_PATH)
        .args(&NGINX_STOP_ARGS)
        .current_dir(&NGINX_CWD)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    thread::sleep(Duration::from_secs(10));
    while !threads.iter().all(|t| t.is_finished()) {}
}
