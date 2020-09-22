mod notifier;
mod service;
use std::env;
use whoami;

static IPS: &'static [&str] = &[
    //"192.168.2.147:9999", // Clab-room-light
    "192.168.86.28:9999", // SC8200-NSLEDS4
    "192.168.86.31:9999", // SC6800-NSLEDS1
];

struct Cleanup;

impl Drop for Cleanup {
    fn drop(&mut self) {
        let icon = ":black_circle:";
        match notifier::slack::send_message("@snaphat", &format!("{}{} smartplug_notifier stopped! {}{} ", icon, icon, icon, icon)) {
            Ok(_) => (),
            Err(err) => println!("Error message: {:?}", err),
        }
    }
}

fn add_hosts() -> notifier::Hosts {
    let icon = ":white_circle:";
    match notifier::slack::send_message("@snaphat", &format!("{}{} smartplug_notifier started! {}{} ", icon, icon, icon, icon)) {
        Ok(_) => (),
        Err(err) => println!("Error message: {:?}", err),
    }

    let mut hosts = notifier::Hosts::new();

    // Add IPs to notifier.
    for ip in IPS.iter() {
        hosts.add(ip);
    }
    hosts
}

#[cfg(windows)]
fn main() {
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("--register") => {
            // Register
            println!("Registering smartplug_slack_notifier service...");
            if let Err(_e) = service::register("smartplug_slack_notifier") {
                println!("Error message: {:?}", _e)
            }
        },
        Some("--unregister") => {
            // Register
            println!("Unregistering smartplug_slack_notifier service...");
            if let Err(_e) = service::unregister("smartplug_slack_notifier") {
                println!("Error message: {:?}", _e)
            }
        },
        Some("--run") => {
            // Run
            if whoami::username() == "SYSTEM" { // Must run as SYSTEM user
                let _cleanup = Cleanup;
                let mut hosts = add_hosts();
                if let Err(_e) = service::run(move || hosts.check()) {
                    println!("Error message: {:?}", _e)
                }
            } else {
                println!("smartplug_slack_notifier service must be run as the SYSTEM user!");
            }
        },
        Some(_) | None => println!("smartplug_slack_notifier.exe [--register] [--unregister] [--run]"),
    }
}

#[cfg(not(windows))]
fn main() {
    let mut hosts = add_hosts();
    loop {
        check_hosts(&mut hosts);
    }
}
