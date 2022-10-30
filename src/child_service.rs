use std::{ffi::OsString, thread::{self, JoinHandle}, time::Duration, sync::{atomic::{AtomicBool, Ordering}, Arc}};

use windows_service::{
    service::{Service, ServiceAccess, ServiceState, ServiceStatus},
    service_manager::{ServiceManager, ServiceManagerAccess},
};

use crate::logger::log;

pub const APACHE_SERVICE_NAME: &str = "APPRO_Apache";
pub const MYSQL_SERVICE_NAME: &str = "APPRO_MySQL";

pub struct ChildServiceControl {
    pub name: String,
    _request_access: ServiceManagerAccess,
    _service_access: ServiceAccess,
    service: Service,
}

impl ChildServiceControl {
    pub fn new(name: &str) -> windows_service::Result<ChildServiceControl> {
        let service_manager =
            ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT).unwrap();
        let service_access = ServiceAccess::QUERY_STATUS
            | ServiceAccess::START
            | ServiceAccess::STOP
            | ServiceAccess::PAUSE_CONTINUE;
        let service = service_manager.open_service(name, service_access)?;

        Ok(ChildServiceControl {
            name: name.to_string(),
            _request_access: ServiceManagerAccess::CONNECT,
            _service_access: service_access,
            service: service,
        })
    }

    pub fn start(&mut self) -> windows_service::Result<()> {
        let service_status = self.service.query_status()?;
        if service_status.current_state != ServiceState::Running {
            self.service.start(&Vec::<OsString>::new())?;
            thread::sleep(Duration::from_secs(1));
        }

        Ok(())
    }

    pub fn stop(&mut self) -> windows_service::Result<()> {
        let service_status = self.service.query_status()?;
        if service_status.current_state != ServiceState::Stopped {
            self.service.stop()?;
            thread::sleep(Duration::from_secs(1));
        }

        Ok(())
    }

    pub fn pause(&mut self) -> windows_service::Result<()> {
        let service_status = self.service.query_status()?;
        if service_status.current_state != ServiceState::Paused {
            self.service.pause()?;
            thread::sleep(Duration::from_secs(1));
        }

        Ok(())
    }

    pub fn resume(&mut self) -> windows_service::Result<()> {
        let service_status = self.service.query_status()?;
        if service_status.current_state != ServiceState::Running {
            self.service.resume()?;
            thread::sleep(Duration::from_secs(1));
        }

        Ok(())
    }

    pub fn status(&mut self) -> windows_service::Result<ServiceStatus> {
        self.service.query_status()
    }
}

fn get_services() -> Vec<ChildServiceControl>{
    
    let mut child_services = vec![];

    let apache = ChildServiceControl::new(APACHE_SERVICE_NAME);
    if apache.is_ok() {
        child_services.push(apache.unwrap());
    }

    let mysql = ChildServiceControl::new(MYSQL_SERVICE_NAME);
    if mysql.is_ok() {
        child_services.push(mysql.unwrap());
    }

    child_services
}

pub fn run_services(
    exit_flag: &Arc<AtomicBool>,
) -> Vec<JoinHandle<()>> {
    let list = get_services();
    let mut threads = Vec::<JoinHandle<()>>::new();

    for mut serv in list {
        let exit_flag = exit_flag.clone();

        threads.push(thread::spawn(move || {
            match serv.start() {
                Ok(_) => log!("{} started", &serv.name),
                Err(err) => log!("{:?}", &err),
            };

            loop {
                if exit_flag.load(Ordering::Relaxed) == true {
                    log!("Stopping: {:?}", &serv.name);
                    match serv.stop() {
                        Ok(_) => log!("{} stopped", &serv.name),
                        Err(err) => log!("{:?}", &err),
                    };
                    break;
                }

                match serv.status() {
                    Ok(status) => {
                        if status.current_state != ServiceState::Running {
                            log!(
                                "Restarting service {}: {:?}",
                                &serv.name,
                                status.current_state
                            );
                            match serv.start() {
                                Ok(()) => log!("Service {} restarted", &serv.name),
                                Err(err) => log!("Can't restart service {}: {}", &serv.name, err),
                            };
                        }
                    }
                    Err(err) => log!("Can't get status for service {}: {}", &serv.name, err),
                };

                thread::sleep(Duration::from_millis(100));
            }
        }));
    }

    threads
}