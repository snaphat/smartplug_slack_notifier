use hs100api::SmartPlug;
use slack_api::sync as slack;
use std::thread;
use std::time::Duration;

// Send slack message
fn send_slack_message(channel: &str, text: &str) {
    let send_msg = || -> Result<(), Box<dyn std::error::Error>> {
        let token = "***REMOVED***";
        let client = slack::default_client().map_err(|_| "Could not get default_client")?;

        slack::chat::post_message(
            &client,
            &token,
            &slack::chat::PostMessageRequest {
                channel: channel,
                text: text,
                username: Some("smartplug_notifier"),
                ..slack::chat::PostMessageRequest::default()
            },
        )?;

        Ok(())
    };

    if let Err(err) = send_msg() {
        println!("Error message: {:?}", err);
    }
}

struct Host {
    ip: String,
    plug: SmartPlug,
    state: Option<i64>,
    alias: String,
}

fn main() {
    // Add hosts to query:
    let ips = [
        //"192.168.2.147:9999", // Clab-room-light
        "192.168.2.146:9999", // SC6800-NSLEDS1
        "192.168.2.101:9999", // SC6800-NSLEDS3
    ];

    // Populate plugs
    let mut hosts: Vec<Host> = Vec::new();
    for ip in &ips {
        hosts.push(Host {
            ip: ip.to_string(),
            state: None, // empty state.
            plug: SmartPlug::new(ip),
            alias: String::from("Unk"),
        });
    }

    // Loop forever
    loop {
        // Loop each plug
        for host in &mut hosts {
            // Query plug info or None on error
            let info = match host.plug.sysinfo() {
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
                host.alias = alias;
            }

            // Insert current state and return old state.
            let old_state = host.state.replace(state).unwrap_or(state);

            // Whether a state change has occurred.
            let changed = old_state != state;

            // If state change send message.
            if changed {
                // Populate info for printing.
                let (state, icon) = match state {
                    1 => ("On", ":red_circle:"),
                    0 => ("Off", ":large_blue_circle:"),
                    _ => ("Err", ":warning:"), // Error state.
                };

                // Format string.
                let msg = format!("{}{} *{}* switched *{}*! {}{}", icon, icon, host.alias, state, icon, icon);

                // Send message.
                send_slack_message("#remote_operation", &msg);
            }

            // Print device state
            println!(
                "Host: {}, Alias: {}, MAC: {}, Relay State: {}, Changed {}",
                host.ip, host.alias, mac, state, changed
            );
        }
        // Sleep before querying again.
        thread::sleep(Duration::from_secs(30));
        println!("========");
    }
}
