use hs100api::SmartPlug;
use std::thread;
use std::time::Duration;

pub mod slack;

struct Host {
    ip: String,
    alias: String,
    state: Option<i64>,
    changed: bool,
    plug: SmartPlug,
}

impl Host {
    // Check plug status.
    pub fn check(&mut self) {
        // Query plug info or None on error
        let info = match self.plug.sysinfo() {
            Ok(info) => info,
            Err(_err) => (hs100api::types::PlugInfo { system: None, emeter: None }),
        };

        // Retrieve info from the plug or -1 on error.
        let (alias, mac, state) = match info.system {
            Some(sys) => (Some(sys.get_sysinfo.alias), sys.get_sysinfo.mac, sys.get_sysinfo.relay_state),
            None => (None, String::from("Unk"), -1),
        };

        // Insert alias if it exists otherwise use original (remembers alias in the event of error).
        if let Some(alias) = alias {
            self.alias = alias;
        }

        // Insert current state and return old state.
        let old_state = self.state.replace(state).unwrap_or(state);

        // Whether a state change has occurred.
        self.changed = (old_state != state) | self.changed;

        // Print device state
        println!(
            "Host: {}, Alias: {}, MAC: {}, Relay State: {}, Changed {}",
            self.ip, self.alias, mac, state, self.changed
        );

        // If state change send message.
        if self.changed {
            // Populate info for printing.
            let (state, icon) = match state {
                1 => ("On", ":red_circle:"),
                0 => ("Off", ":large_blue_circle:"),
                _ => ("Err", ":warning:"), // Error state.
            };

            // Format string.
            let msg = format!("{}{} *{}* switched *{}*! {}{}", icon, icon, self.alias, state, icon, icon);

            // Send message.
            match slack::send_message("#remote_operation", &msg) {
                Ok(_) => self.changed = false, // switch changed state if successfully sent.
                Err(err) => println!("Error message: {:?}", err),
            }
        }
    }
}

#[derive(Default)]
pub struct Hosts {
    hosts: Vec<Host>,
}

impl Hosts {
    pub fn new() -> Hosts {
        Hosts { ..Default::default() }
    }

    // Add plug.
    pub fn add(&mut self, ip: &'static str) {
        self.hosts.push(Host {
            ip: ip.to_string(),
            alias: String::from("Unk"),
            state: None, // empty state.
            changed: false,
            plug: SmartPlug::new(ip),
        });
    }

    // Check hosts.
    pub fn check(&mut self) {
        for host in &mut self.hosts {
            host.check();
            println!("========");
            thread::sleep(Duration::from_secs(2));
        }
    }
}
