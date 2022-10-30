use chrono::Utc;
use core::fmt::Display;
use std::fs::OpenOptions;
use std::io::prelude::*;

pub fn log<T: Display + ?Sized>(message: &T) {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open("servicers.log")
        .unwrap();

    let time = Utc::now().format("%F %T").to_string();

    writeln!(file, "[{}]: {}", time, message).unwrap();
}
