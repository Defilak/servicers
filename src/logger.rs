use chrono::Utc;
use core::fmt::Display;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::sync::Mutex;

static WRITE_CHECK: Mutex<bool> = Mutex::new(true);

pub fn log_write<T: Display + ?Sized>(message: &T) {
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

macro_rules! log {
    () => {};
    ($($arg:tt)*) => {{
        use crate::logger::log_write;
        let text = format!($($arg)*);
        log_write(&text);
    }};
}

pub(crate) use log;