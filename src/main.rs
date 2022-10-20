use std::process::Child;
use std::process::Command;
use std::thread;
use std::thread::JoinHandle;
use std::sync::Mutex;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
/*
struct ChildProcess {
    program: String,
    args: Vec<String>,
    workdir: Option<String>,
    child: Option<Child>,
}

impl Drop for ChildProcess {
    fn drop(&mut self) {
        self.child.as_mut().unwrap().kill();
    }
}

impl ChildProcess {
    fn new(program: String, args: Vec<String>) -> ChildProcess {
        ChildProcess {
            program,
            args,
            workdir: None,
            child: None,
        }
    }

    fn start(&mut self) {
        self.child = match Command::new(&self.program).args(&self.args).spawn() {
            Ok(child) => Some(child),
            Err(err) => panic!("{:?}", err),
        };
    }

    fn autorestart(&mut self, shared: Arc<AtomicBool>) {
        loop {
            match self.child.as_mut().unwrap().try_wait() {
                Ok(Some(status)) => self.start(),
                Ok(None) => {
                }
                Err(e) => self.start(),
            };



            /*match self.child.as_mut().unwrap().wait() {
                Ok(ok) => println!("Процесс завершился с кодом {:?}", ok),
                Err(err) => println!(
                    "Процесс завершился с ошибкой: {:?}. Его ждет перезапуск.",
                    err
                ),
            };
            self.start();*/
        }
    }
}

fn main() {
    //let mut childs = vec![make_child(), make_child1()];

    let list = vec![
        ChildProcess::new("php.exe".to_string(), vec!["C:/Users/defilak/Desktop/rust/servicers/test/app1.php".to_string()]),
        ChildProcess::new(
            "php.exe".to_string(),
            vec!["-S".to_string(), "localhost:8080".to_string()],
        ),
    ];

    //let arc = Arc::new(AtomicBool);//true is working
    let atom = Arc::new(AtomicBool::new(true));

    let mut threads = Vec::<JoinHandle<()>>::new();
    for mut proc in list {
        let shared = atom.clone();
        threads.push(thread::spawn(move || {
            proc.start();
            proc.autorestart(shared);
        }));
    }

    loop {
        if threads.iter().all(|t| t.is_finished()) {
            println!("all threads gone");
            break;
        }
        thread::sleep(std::time::Duration::from_millis(100));

        thread::sleep(std::time::Duration::from_secs(20));

        break;
    }
}

*/
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
