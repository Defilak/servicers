use std::process::Child;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

struct ChildProcess {
    program: String,
    args: Vec<String>,
    workdir: Option<String>,
    child: Option<Child>,
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

    fn autorestart(&mut self, need_exit: Arc<AtomicBool>) {
        loop {
            match self.child.as_mut().unwrap().try_wait() {
                Ok(Some(status)) => self.start(),
                Ok(None) => {}
                Err(e) => self.start(),
            };

            if need_exit.load(Ordering::Relaxed) {
                self.child.as_mut().unwrap().kill();
                println!("kill proc");
                break;
            }

            thread::sleep(Duration::from_millis(100));
        }
    }
}

fn main() {
    //let mut childs = vec![make_child(), make_child1()];

    let list = vec![
        ChildProcess::new(
            "php.exe".to_string(),
            vec!["D:/projects/servicers-main/test/app1.php".to_string()],
        ),
        ChildProcess::new(
            "php.exe".to_string(),
            vec!["-S".to_string(), "localhost:8080".to_string()],
        ),
    ];

    // Атомарный потокобезопасный флажок обернутый в потокобезопасный strong счетчик ссылок.
    // Видимо, подразумевается что он безопасно чистит память при выходе из блока. Интересно как.
    // От родителя к потомку - Arc, обратно Weak. Написано, что иначе память потечет.
    let need_exit_flag = Arc::new(AtomicBool::new(false));

    let mut threads = Vec::<JoinHandle<()>>::new();
    for mut proc in list {
        // Для каждого копирую ссылку
        let shared = need_exit_flag.clone();
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
        need_exit_flag.store(true, Ordering::Relaxed);
    }
}
