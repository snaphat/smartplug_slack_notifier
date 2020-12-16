mod notifier;
mod service;
use serde::{Deserialize, Serialize};
use std::{env, fs::File, io::BufReader, io::Error};
use whoami;

//"192.168.2.147:9999", // Clab-room-light
//"192.168.86.28:9999", // SC8200-NSLEDS4
//"192.168.86.31:9999", // SC6800-NSLEDS1
//"192.168.86.108:9999", // X6901sc(170) - HDILED2
//"192.168.86.109:9999", // X6901sc(178) - NSLEDS3

#[derive(Serialize, Deserialize)]
struct Config {
    token: String,
    hosts: Vec<String>,
}

struct Cleanup {
    token: String,
}

impl Cleanup {
    pub fn new(token: String) -> Cleanup {
        Cleanup { token }
    }
}

impl Drop for Cleanup {
    fn drop(&mut self) {
        let icon = ":black_circle:";
        match notifier::slack::send_message(&self.token, "@snaphat", &format!("{}{} smartplug_notifier stopped! {}{} ", icon, icon, icon, icon)) {
            Ok(_) => (),
            Err(err) => println!("Error sending slack message: {:?}", err),
        }
    }
}

fn load_config() -> Result<Config, Error> {
    let mut path = env::current_exe()?;
    path.pop();
    path.push("config.json");
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let config: Config = serde_json::from_reader(reader)?;
    Ok(config)
}

fn add_hosts() -> notifier::Hosts {

    if let Ok(config) = load_config() {
        let mut hosts = notifier::Hosts::new(config.token.clone());
        let icon = ":white_circle:";
        match notifier::slack::send_message(
            &config.token,
            "@snaphat",
            &format!("{}{} smartplug_notifier started! {}{} ", icon, icon, icon, icon),
        ) {
            Ok(_) => (),
            Err(err) => println!("Error sending slack message: {:?}", err),
        }

        // Add IPs to notifier.
        for ip in config.hosts.iter() {
            hosts.add(ip.clone());
        }
        hosts
    }
    else {
        notifier::Hosts::new("".to_string())
    }
}

#[cfg(windows)]
fn main() {
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("--register") => {
            // Register
            println!("Registering smartplug_slack_notifier service...");
            if let Err(_e) = service::register("smartplug_slack_notifier") {
                println!("Error registering windows service: {:?}", _e)
            }
        }
        Some("--unregister") => {
            // Register
            println!("Unregistering smartplug_slack_notifier service...");
            if let Err(_e) = service::unregister("smartplug_slack_notifier") {
                println!("Error unregistering windows service: {:?}", _e)
            }
        }
        Some("--run") => {
            // Run
            if whoami::username() == "SYSTEM" {
                // Must run as SYSTEM user
                let mut hosts = add_hosts();
                let _cleanup = Cleanup::new(hosts.token.clone());
                if let Err(_e) = service::run(move || hosts.check()) {
                    println!("Error running service: {:?}", _e)
                }
            } else {
                println!("smartplug_slack_notifier service must be run as the SYSTEM user!");
            }
        }
        Some("--debug") => {
            // Run
            let mut hosts = add_hosts();
            let _cleanup = Cleanup::new(hosts.token.clone());
            loop {
                hosts.check();
            }
        }
        Some(_) | None => println!("smartplug_slack_notifier.exe [--register] [--unregister] [--run] [--debug]"),
    }
}

#[cfg(not(windows))]
fn main() {
    let mut hosts = add_hosts();
    let _cleanup = Cleanup::new(hosts.token.clone());
    loop {
        hosts.check();
    }
}
