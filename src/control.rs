use std::{
    ffi::{OsStr, OsString},
    thread,
    time::Duration,
};
use windows_service::{
    service::{
        Service, ServiceAccess, ServiceErrorControl, ServiceInfo, ServiceStartType, ServiceState,
        ServiceType,
    },
    service_manager::{ServiceManager, ServiceManagerAccess},
};

const SERVICE_DISPLAY_NAME: &str = "A1 Супервизор АП-ПРО";
const SERVICE_DESC: &str = "Контроль сервисов АП";

pub struct ServiceControl {
    pub name: String,
    _request_access: ServiceManagerAccess,
    _service_access: ServiceAccess,
    service: Service,
}

impl ServiceControl {
    pub fn new(name: &str) -> windows_service::Result<ServiceControl> {
        let service_manager =
            ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT).unwrap();
        let service_access =
            ServiceAccess::QUERY_STATUS | ServiceAccess::START | ServiceAccess::STOP;
        let service = service_manager.open_service(name, service_access)?;

        Ok(ServiceControl {
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
        let service = get_service(ServiceManagerAccess::CONNECT, ServiceAccess::PAUSE_CONTINUE)?;

        let service_status = service.query_status()?;
        if service_status.current_state != ServiceState::Paused {
            service.pause()?;
            thread::sleep(Duration::from_secs(1));
        }

        Ok(())
    }

    pub fn resume() -> windows_service::Result<()> {
        let service = get_service(ServiceManagerAccess::CONNECT, ServiceAccess::PAUSE_CONTINUE)?;

        let service_status = service.query_status()?;
        if service_status.current_state != ServiceState::Running {
            service.resume()?;
            thread::sleep(Duration::from_secs(1));
        }

        Ok(())
    }

    pub fn status() -> windows_service::Result<()> {
        let service = get_service(ServiceManagerAccess::CONNECT, ServiceAccess::INTERROGATE)?;

        let service_status = service.query_status()?;
        println!("{:?}", service_status);

        Ok(())
    }
}

pub fn get_service(
    request_access: ServiceManagerAccess,
    service_access: ServiceAccess,
) -> windows_service::Result<Service> {
    let service_manager = ServiceManager::local_computer(None::<&str>, request_access)?;

    service_manager.open_service(
        super::SERVICE_NAME,
        ServiceAccess::QUERY_STATUS | service_access,
    )
}

pub fn start() -> windows_service::Result<()> {
    let service = get_service(ServiceManagerAccess::CONNECT, ServiceAccess::START)?;

    let service_status = service.query_status()?;
    if service_status.current_state != ServiceState::Running {
        service.start(&[OsStr::new("runservice")])?;
        // Wait for service to stop
        thread::sleep(Duration::from_secs(1));
    }

    Ok(())
}

pub fn stop() -> windows_service::Result<()> {
    let service = get_service(ServiceManagerAccess::CONNECT, ServiceAccess::STOP)?;

    let service_status = service.query_status()?;
    if service_status.current_state != ServiceState::Stopped {
        service.stop()?;
        // Wait for service to stop
        thread::sleep(Duration::from_secs(1));
    }

    Ok(())
}

pub fn pause() -> windows_service::Result<()> {
    let service = get_service(ServiceManagerAccess::CONNECT, ServiceAccess::PAUSE_CONTINUE)?;

    let service_status = service.query_status()?;
    if service_status.current_state != ServiceState::Paused {
        service.pause()?;
        // Wait for service to stop
        thread::sleep(Duration::from_secs(1));
    }

    Ok(())
}

pub fn resume() -> windows_service::Result<()> {
    let service = get_service(ServiceManagerAccess::CONNECT, ServiceAccess::PAUSE_CONTINUE)?;

    let service_status = service.query_status()?;
    if service_status.current_state != ServiceState::Running {
        service.resume()?;
        // Wait for service to stop
        thread::sleep(Duration::from_secs(1));
    }

    Ok(())
}

pub fn status() -> windows_service::Result<()> {
    let service = get_service(ServiceManagerAccess::CONNECT, ServiceAccess::INTERROGATE)?;

    let service_status = service.query_status()?;
    println!("{:?}", service_status);

    Ok(())
}

pub fn install() -> windows_service::Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_info = ServiceInfo {
        name: OsString::from(super::SERVICE_NAME),
        display_name: OsString::from(SERVICE_DISPLAY_NAME),
        service_type: ServiceType::OWN_PROCESS,
        start_type: ServiceStartType::AutoStart,
        error_control: ServiceErrorControl::Normal,
        executable_path: std::env::current_exe().unwrap(),
        launch_arguments: vec![OsString::from("runservice")],
        dependencies: vec![],
        account_name: None, // run as System
        account_password: None,
    };
    let service = service_manager.create_service(&service_info, ServiceAccess::CHANGE_CONFIG)?;
    service.set_description(SERVICE_DESC)?;
    //start()?;
    Ok(())
}

pub fn uninstall() -> windows_service::Result<()> {
    let manager_access = ServiceManagerAccess::CONNECT;
    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

    let service_access = ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE;
    let service = service_manager.open_service(super::SERVICE_NAME, service_access)?;

    let service_status = service.query_status()?;
    if service_status.current_state != ServiceState::Stopped {
        service.stop()?;
        // Wait for service to stop
        thread::sleep(Duration::from_secs(1));
    }

    service.delete()?;
    Ok(())
}

/*
struct ServiceControl {
    manager: ServiceManager,
    service: Service,
}

impl ServiceControl {
    fn new(name: &str)/*  -> ServiceControl */{
        //ServiceControl { manager, service }
        let manager = ServiceManager::local_computer(None::<&str>, request_access);
    }

    fn get_service(
        request_access: ServiceManagerAccess,
        service_access: ServiceAccess,
    ) -> windows_service::Result<Service> {
        let service_manager = ServiceManager::local_computer(None::<&str>, request_access)?;

        service_manager.open_service("rtest", ServiceAccess::QUERY_STATUS | service_access)
    }

    fn install() -> windows_service::Result<()> {
        let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
        let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;

        // This example installs the service defined in `examples/ping_service.rs`.
        // In the real world code you would set the executable path to point to your own binary
        // that implements windows service.
        let service_binary_path = ::std::env::current_exe().unwrap();

        let service_info = ServiceInfo {
            name: OsString::from("rtest"),
            display_name: OsString::from("A1 My test service"),
            service_type: ServiceType::OWN_PROCESS,
            start_type: ServiceStartType::AutoStart,
            error_control: ServiceErrorControl::Normal,
            executable_path: service_binary_path,
            launch_arguments: vec![],
            dependencies: vec![],
            account_name: None, // run as System
            account_password: None,
        };
        let service =
            service_manager.create_service(&service_info, ServiceAccess::CHANGE_CONFIG)?;
        service.set_description("Windows service example from windows-service-rs")?;
        Ok(())
    }
    fn uninstall() {}
    fn start() {}
    fn stop() {}
    fn pause() {}
    fn resume() {}
    fn status() {}
}*/
