use std::process::Child;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::{
    ffi::OsString,
    net::{IpAddr, SocketAddr, UdpSocket},
    sync::mpsc,
    time::Duration,
};

use windows_service::{
    define_windows_service,
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult, ServiceStatusHandle},
    service_dispatcher, Result,
};

const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

const LOOPBACK_ADDR: [u8; 4] = [127, 0, 0, 1];
const RECEIVER_PORT: u16 = 1234;
const PING_MESSAGE: &str = "ping\n";
const PAUSED_MESSAGE: &str = "paused\n";
const RESUME_MESSAGE: &str = "RESUME_MESSAGE\n";
const STOP_MESSAGE: &str = "STOP_MESSAGE\n";
const ITTER_MESSAGE: &str = "ITTER_MESSAGE\n";

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

enum StatusMessage<T> {
    Interrogate(T),
    Continue(T),
    Pause(T),
    Stop(T),
}
struct ChildProcess {
    program: String,
    args: Vec<String>,
    workdir: Option<String>,
    child: Option<Child>,
}

impl ChildProcess {
    fn new(program: String, args: Vec<String>) -> ChildProcess {
        ChildProcess {
            program,
            args,
            workdir: None,
            child: None,
        }
    }

    fn start(&mut self) {
        self.child = match Command::new(&self.program).args(&self.args).spawn() {
            Ok(child) => Some(child),
            Err(err) => panic!("{:?}", err),
        };
    }

    fn autorestart(&mut self, need_exit: Arc<AtomicBool>) {
        loop {
            match self.child.as_mut().unwrap().try_wait() {
                Ok(Some(status)) => self.start(),
                Ok(None) => {}
                Err(e) => self.start(),
            };

            if need_exit.load(Ordering::Relaxed) {
                self.child.as_mut().unwrap().kill();
                println!("kill proc");
                break;
            }

            thread::sleep(Duration::from_millis(100));
        }
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

    run_main_loop(status_handle, shutdown_rx);

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
    let list = vec![
        ChildProcess::new(
            "php.exe".to_string(),
            vec!["C:/Users/defilak/Desktop/rust/servicers/test/app1.php".to_string()],
        ),
        ChildProcess::new(
            "php.exe".to_string(),
            vec!["-S".to_string(), "localhost:8080".to_string()],
        ),
    ];

    // Атомарный потокобезопасный флажок обернутый в потокобезопасный strong счетчик ссылок.
    // Видимо, подразумевается что он безопасно чистит память при выходе из блока. Интересно как.
    // От родителя к потомку - Arc, обратно Weak. Написано, что иначе память потечет.
    let need_exit_flag = Arc::new(AtomicBool::new(false));

    let mut threads = Vec::<JoinHandle<()>>::new();
    for mut proc in list {
        // Для каждого копирую ссылку
        let shared = need_exit_flag.clone();
        threads.push(thread::spawn(move || {
            proc.start();
            proc.autorestart(shared);
        }));
    }

    loop {
        if threads.iter().all(|t| t.is_finished()) {
            println!("all threads gone");
            break;
        }

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
                    //threads.iter().all(|t| t.);
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
                },
                _ => ()
            },
            // Break the loop either upon stop or channel disconnect
            Err(mpsc::RecvTimeoutError::Disconnected) => break,

            // Continue work if no events were received within the timeout
            Err(mpsc::RecvTimeoutError::Timeout) => (),
        }
    }

    // For demo purposes this service sends a UDP packet once a second.
    /*let loopback_ip = IpAddr::from(LOOPBACK_ADDR);
    let sender_addr = SocketAddr::new(loopback_ip, 0);
    let receiver_addr = SocketAddr::new(loopback_ip, RECEIVER_PORT);
    //let msg = PING_MESSAGE.as_bytes();

    let mut msg = PING_MESSAGE.as_bytes();
    let socket = UdpSocket::bind(sender_addr).unwrap();

    loop {
        let _ = socket.send_to(msg, receiver_addr);

        // Poll shutdown event.
        match shutdown_rx.recv_timeout(Duration::from_secs(1)) {
            Ok(var) => match var {
                StatusMessage::Interrogate(text) => {
                    msg = text.as_bytes();
                }
                StatusMessage::Continue(text) => {
                    msg = text.as_bytes();
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
                StatusMessage::Pause(text) => {
                    msg = text.as_bytes();
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
                StatusMessage::Stop(text) => {
                    msg = text.as_bytes();
                }
            },
            // Break the loop either upon stop or channel disconnect
            Err(mpsc::RecvTimeoutError::Disconnected) => break,

            // Continue work if no events were received within the timeout
            Err(mpsc::RecvTimeoutError::Timeout) => (),
        };
    }*/

    Ok(())
}
