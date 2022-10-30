use chrono::Utc;
use core::fmt::Debug;
use core::fmt::Display;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::sync::Mutex;

static WRITE_CHECK: Mutex<bool> = Mutex::new(true);

pub fn log<T: Display + ?Sized>(message: &T) {
    println!("{}", &message);

    let mut num = WRITE_CHECK.lock().unwrap();
    if num.eq(&true) {
        *num = false;

        let mut file_path = std::env::current_exe().unwrap();
        file_path.pop();
        file_path.push("servicers.log");

        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(file_path)
            .unwrap();

        let time = Utc::now().format("%F %T").to_string();

        writeln!(file, "[{}]: {}", time, message).unwrap();
        *num = true;
    }
}
