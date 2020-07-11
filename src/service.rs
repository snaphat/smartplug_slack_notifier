#[cfg(windows)]
use std::{ffi::OsString, sync::mpsc, time::Duration};
use windows_service::{
    define_windows_service,
    service::{ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus, ServiceType},
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher, Result,
};

use std::thread;

const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;
static SERVICE_NAME: &str = "smartplug_slack_notifier";

//fn unregister(name: &str) -> windows_service::Result<()> {
//    let manager_access = ServiceManagerAccess::CONNECT;
//    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;
//
//    let service_access = ServiceAccess::QUERY_STATUS | ServiceAccess::STOP | ServiceAccess::DELETE;
//    let service = service_manager.open_service(name, service_access)?;
//
//    let service_status = service.query_status()?;
//    if service_status.current_state != ServiceState::Stopped {
//        service.stop()?;
//        // Wait for service to stop
//        thread::sleep(Duration::from_secs(1));
//    }
//
//    service.delete()?;
//    Ok(())
//}
//
//pub fn register(name: &str) -> windows_service::Result<()> {
//    let manager_access = ServiceManagerAccess::CONNECT | ServiceManagerAccess::CREATE_SERVICE;
//    let service_manager = ServiceManager::local_computer(None::<&str>, manager_access)?;
//
//    // This example installs the service defined in `examples/ping_service.rs`.
//    // In the real world code you would set the executable path to point to your own binary
//    // that implements windows service.
//    let service_binary_path = ::std::env::current_exe().unwrap().with_file_name("smartplug_slack_notifier.exe");
//    let service_info = ServiceInfo {
//        name: OsString::from(name),
//        display_name: OsString::from(name),
//        service_type: ServiceType::OWN_PROCESS,
//        start_type: ServiceStartType::OnDemand,
//        error_control: ServiceErrorControl::Normal,
//        executable_path: service_binary_path,
//        launch_arguments: vec![],
//        dependencies: vec![],
//        account_name: None, // run as System
//        account_password: None,
//    };
//    let _service = service_manager.create_service(&service_info, ServiceAccess::empty())?;
//    Ok(())
//}

static mut HANDLER: Option<Box<dyn FnMut() + Send>> = None;
pub fn run(handler: impl FnMut() + Send + 'static) -> Result<()> {
    unsafe {
        HANDLER = Some(Box::new(handler));
    }
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)
}

// Generate the windows service boilerplate.
// The boilerplate contains the low-level service entry function (ffi_service_main) that parses
// incoming service arguments into Vec<OsString> and passes them to user defined service
// entry (my_service_main).
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
    // Create a channel to be able to poll a stop event from the service worker loop.
    let (shutdown_tx, shutdown_rx) = mpsc::channel();

    // Define system service event handler that will be receiving service events.
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            // Notifies a service to report its current status information to the service
            // control manager. Always return NoError even if not implemented.
            ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,

            // Handle stop
            ServiceControl::Stop => {
                shutdown_tx.send(()).unwrap();
                ServiceControlHandlerResult::NoError
            }

            _ => ServiceControlHandlerResult::NotImplemented,
        }
    };

    // Register system service event handler.
    // The returned status handle should be used to report service status changes to the system.
    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

    // Tell the system that service is running
    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::from_secs(30),
        process_id: None,
    })?;

    loop {
        let check = unsafe {
            match &mut HANDLER {
                Some(a) => Some(thread::spawn(a)),
                None => None,
            }
        };

        // Poll shutdown event.
        match shutdown_rx.recv_timeout(Duration::from_secs(2)) {
            // Break the loop either upon stop or channel disconnect
            Ok(_) | Err(mpsc::RecvTimeoutError::Disconnected) => break,

            // Continue work if no events were received within the timeout
            Err(mpsc::RecvTimeoutError::Timeout) => (),
        };

        if let Some(check) = check {
            if let Err(_) = check.join() {}
        }
    }

    // Tell the system that service has stopped.
    status_handle.set_service_status(ServiceStatus {
        service_type: SERVICE_TYPE,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::from_secs(30),
        process_id: None,
    })?;

    Ok(())
}
