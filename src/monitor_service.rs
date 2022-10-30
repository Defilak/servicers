use std::fs::File;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::{ffi::OsString, sync::mpsc, time::Duration};
use windows_service::{
    define_windows_service,
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult, ServiceStatusHandle},
    service_dispatcher, Result,
};

use crate::child_proc::ChildProcess;
use crate::proc_config::{self, ProcessConfig};
use std::env::current_dir;
use std::io::Write;

const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

pub fn run() -> Result<()> {
    // Register generated `ffi_service_main` with the system and start the service, blocking
    // this thread until the service is stopped.
    service_dispatcher::start(super::SERVICE_NAME, ffi_service_main)
}

// Generate the windows service boilerplate.
define_windows_service!(ffi_service_main, my_service_main);

// Service entry function which is called on background thread by the system with service
// parameters. There is no stdout or stderr at this point so make sure to configure the log
// output to file if needed.
pub fn my_service_main(_arguments: Vec<OsString>) {
    if let Err(_e) = run_service() {
        // Handle the error, by logging or something.
    }
}

pub fn run_service() -> Result<()> {
    // Канал (видимо типа nio в жаббе), чтобы иметь возможность опросить событие остановки из цикла сервисного работника.
    let (shutdown_tx, shutdown_rx) = mpsc::channel();

    // Создаю калбек для обработки событий жизненного цикла службы
    // Из него подаются события из венды в приложение
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            // Уведомляет службу о необходимости сообщить службе информацию о текущем состоянии.
            // диспетчер управления. Всегда возвращайте NoError, даже если это не реализовано.
            ServiceControl::Interrogate => {
                shutdown_tx.send(ServiceControl::Interrogate).unwrap();
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Continue => {
                shutdown_tx.send(ServiceControl::Continue).unwrap();
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Pause => {
                shutdown_tx.send(ServiceControl::Pause).unwrap();
                ServiceControlHandlerResult::NoError
            }
            // Handle stop
            ServiceControl::Stop => {
                shutdown_tx.send(ServiceControl::Stop).unwrap();
                ServiceControlHandlerResult::NoError
            }

            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    // Регистрирую обработчик событий службы.
    // Возвращаемый дескриптор состояния следует использовать для сообщения системе об изменении состояния службы.
    let status_handle = service_control_handler::register(super::SERVICE_NAME, event_handler)?;

    // Сообщаю венде, что служба запущена
    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::PAUSE_CONTINUE,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    match run_main_loop(status_handle, shutdown_rx) {
        Err(err) => {
            let file = File::create(current_dir().unwrap().join("errors.log"));
            match file.unwrap().write(err.to_string().as_bytes()) {
                _ => panic!("pizdec!"),
            }
        }
        Ok(_e) => (),
    };

    // Tell the system that service has stopped.
    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    Ok(())
}

fn run_main_loop(
    status_handle: ServiceStatusHandle,
    shutdown_rx: Receiver<ServiceControl>,
) -> windows_service::Result<()> {
    let mut list = Vec::<ChildProcess>::new();
    for cfg in proc_config::load() {
        list.push(ChildProcess::from_config(cfg));
    }

    // Атомарный потокобезопасный флажок обернутый в потокобезопасный strong счетчик ссылок.
    // Видимо, подразумевается что он безопасно чистит память при выходе из блока. Интересно как.
    // От родителя к потомку - Arc, обратно Weak. Написано, что иначе память потечет.
    let need_exit_flag = Arc::new(AtomicBool::new(false));
    let threads = run_processes(list, &need_exit_flag);

    loop {
        match shutdown_rx.recv_timeout(Duration::from_secs(1)) {
            Ok(var) => match var {
                ServiceControl::Interrogate => {}
                ServiceControl::Continue => {
                    status_handle.set_service_status(ServiceStatus {
                        service_type: SERVICE_TYPE,
                        current_state: ServiceState::Running,
                        controls_accepted: ServiceControlAccept::STOP
                            | ServiceControlAccept::PAUSE_CONTINUE,
                        exit_code: ServiceExitCode::Win32(0),
                        checkpoint: 0,
                        wait_hint: Duration::default(),
                        process_id: None,
                    })?;
                }
                ServiceControl::Pause => {
                    status_handle.set_service_status(ServiceStatus {
                        service_type: SERVICE_TYPE,
                        current_state: ServiceState::Paused,
                        controls_accepted: ServiceControlAccept::STOP
                            | ServiceControlAccept::PAUSE_CONTINUE,
                        exit_code: ServiceExitCode::Win32(0),
                        checkpoint: 0,
                        wait_hint: Duration::default(),
                        process_id: None,
                    })?;
                }
                ServiceControl::Stop => {
                    need_exit_flag.store(true, Ordering::Relaxed);

                    status_handle.set_service_status(ServiceStatus {
                        service_type: SERVICE_TYPE,
                        current_state: ServiceState::StopPending,
                        controls_accepted: ServiceControlAccept::STOP,
                        exit_code: ServiceExitCode::Win32(0),
                        checkpoint: 1,
                        wait_hint: Duration::from_secs(5),
                        process_id: None,
                    })?;

                    while !threads.iter().all(|t| t.is_finished()) {}

                    status_handle.set_service_status(ServiceStatus {
                        service_type: SERVICE_TYPE,
                        current_state: ServiceState::Stopped,
                        controls_accepted: ServiceControlAccept::STOP,
                        exit_code: ServiceExitCode::Win32(0),
                        checkpoint: 2,
                        wait_hint: Duration::default(),
                        process_id: None,
                    })?;
                }
                _ => (),
            },
            // Break the loop either upon stop or channel disconnect
            Err(mpsc::RecvTimeoutError::Disconnected) => break,

            // Continue work if no events were received within the timeout
            Err(mpsc::RecvTimeoutError::Timeout) => (),
        }
    }

    Ok(())
}

pub fn run_processes1(mut list: Vec<ChildProcess>) {
    for proc in &mut list {
        proc.start();
    }

    loop {
        for proc in &mut list {
            if proc.config.is_valid() {
                proc.try_restart();
            }
        }

        thread::sleep(Duration::from_millis(100));
    }
}

#[test]
fn test_run() {
    let mut list = Vec::<ChildProcess>::new();
    for cfg in proc_config::load() {

        list.push(ChildProcess::from_config(cfg));
    }

    run_processes1(list);
}

pub fn run_processes(list: Vec<ChildProcess>, exit_flag: &Arc<AtomicBool>) -> Vec<JoinHandle<()>> {
    let mut threads = Vec::<JoinHandle<()>>::new();
    for mut proc in list {
        // Для каждого копирую ссылку
        let shared = exit_flag.clone();
        threads.push(thread::spawn(move || {
            proc.start();
            loop {
                proc.try_restart();
                if shared.load(Ordering::Relaxed) {
                    proc.kill();
                    println!("kill proc");
                    break;
                }

                thread::sleep(Duration::from_millis(100));
            }
        }));
    }

    threads
}
