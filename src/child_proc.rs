use crate::proc_config::ProcessConfig;
use std::process::Child;

pub struct ChildProcess {
    config: ProcessConfig<'static>,
    child: Option<Child>,
}

impl ChildProcess {
    pub fn _new(
        program: &'static str,
        args: Vec<&'static str>,
        workdir: Option<&'static str>,
    ) -> ChildProcess {
        ChildProcess {
            config: ProcessConfig {
                program: program,
                args: args,
                cwd: workdir.unwrap(),
            },
            child: None,
        }
    }

    pub fn from_config(config: ProcessConfig<'static>) -> ChildProcess {
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
