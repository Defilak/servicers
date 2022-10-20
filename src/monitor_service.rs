use std::sync::mpsc::Receiver;
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
                shutdown_tx
                    .send(StatusMessage::Interrogate(ITTER_MESSAGE))
                    .unwrap();
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Continue => {
                shutdown_tx
                    .send(StatusMessage::Continue(RESUME_MESSAGE))
                    .unwrap();
                ServiceControlHandlerResult::NoError
            }
            ServiceControl::Pause => {
                shutdown_tx
                    .send(StatusMessage::Pause(PAUSED_MESSAGE))
                    .unwrap();
                ServiceControlHandlerResult::NoError
            }
            // Handle stop
            ServiceControl::Stop => {
                shutdown_tx.send(StatusMessage::Stop(STOP_MESSAGE)).unwrap();
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
    shutdown_rx: Receiver<StatusMessage<&str>>,
) -> windows_service::Result<()> {
    // For demo purposes this service sends a UDP packet once a second.
    let loopback_ip = IpAddr::from(LOOPBACK_ADDR);
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
    }

    Ok(())
}
