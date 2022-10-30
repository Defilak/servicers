use crate::proc_config::{ProcessConfig, ProcessConfigState};
use std::process::Child;

pub struct ChildProcess {
    config: ProcessConfig,
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

    pub fn start(&mut self) {
        self.child = match self.config.spawn_new() {
            Ok(child) => {
                dbg!(&child);
                Some(child)
            }
            Err(err) => panic!("{:?}", err),
        };
    }

    pub fn try_restart(&mut self) {
        match self.child.as_mut().unwrap().try_wait() {
            Ok(Some(status)) => {
                dbg!(status);
                self.start()
            }
            Ok(None) => {}
            Err(e) => {
                dbg!(e);
                self.start()
            }
        };
    }

    pub fn kill(&mut self) {
        match self.child.as_mut().unwrap().kill() {
            Ok(e) => {
                dbg!(e);
            }
            Err(err) => {
                dbg!(err);
            }
        }
    }
}
