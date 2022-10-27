use std::process::Child;
use std::process::Command;
use std::process::Stdio;

#[derive(Debug)]
pub struct ProcessConfig<'a> {
    pub program: &'a str,
    pub args: Vec<&'a str>,
    pub cwd: &'a str,
}

impl<'a> ProcessConfig<'a> {
    pub fn _new(program: &str) -> ProcessConfig {
        ProcessConfig {
            program: program,
            args: vec![],
            cwd: "",
        }
    }

    pub fn spawn_new(&self) -> Result<Child, std::io::Error> {
        Command::new(&self.program)
            .args(&self.args)
            .current_dir(&self.cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

pub fn load() -> Vec<ProcessConfig<'static>> {
    vec![
        ProcessConfig {
            program: "C:/nginx/nginx.exe",
            args: vec![],
            cwd: "C:/nginx",
        },
        ProcessConfig {
            program: "C:/php/8.1.8/php-cgi.exe",
            args: vec!["-b", "localhost:9123"],
            cwd: "C:/nginx/html",
        },
    ]
}
