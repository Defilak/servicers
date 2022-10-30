use super::logger::log;
use serde::{self, Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::path::Path;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ProcessConfigState {
    Enabled,
    Disabled,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessConfig {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: String,
    pub state: ProcessConfigState,
}

impl ProcessConfig {
    pub fn _new(program: String, args: Vec<String>, cwd: String) -> ProcessConfig {
        ProcessConfig {
            program: program,
            args: args,
            cwd: cwd,
            state: ProcessConfigState::Enabled,
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

pub fn load() -> Vec<ProcessConfig> {
    let file_path = Path::new("servicers.json");
    if !file_path.exists() {
        create_default();
    }

    let vec: Vec<ProcessConfig> = match File::open(file_path) {
        Ok(file) => {
            let mut reader = BufReader::new(file);

            let mut text = String::new();
            reader.read_to_string(&mut text).unwrap();

            let sad = serde_json::from_str(&text).unwrap();
            sad
        }
        Err(err) => {
            log(&err);
            Vec::<ProcessConfig>::new()
        }
    };

    vec
    /*vec![
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
    ]*/
}

fn save(cfg: Vec<&ProcessConfig>) -> std::io::Result<()> {
    let text = serde_json::to_string_pretty(&cfg)?;

    File::create("servicers.json")?.write_all(&text.as_bytes())?;

    Ok(())
}

fn create_default() {
    save(vec![&ProcessConfig {
        program: "".to_string(),
        args: vec![],
        cwd: "".to_string(),
        state: ProcessConfigState::Enabled,
    }])
    .unwrap();
}

#[test]
fn test_load() {
    dbg!(load());
}