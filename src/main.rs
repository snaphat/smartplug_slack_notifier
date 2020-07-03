use hs100api::SmartPlug;
use slack_api::sync as slack;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

fn send_slack_message(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    let token = "***REMOVED***";
    let client = slack::default_client().map_err(|_| "Could not get default_client")?;

    let response = slack::chat::post_message(
        &client,
        &token,
        &slack::chat::PostMessageRequest {
            channel: "#remote_operation",
            text: text,
            username: Some("smartplug_notifier"),
            ..slack::chat::PostMessageRequest::default()
        },
    );

    if let Ok(response) = response {
        if let Some(message) = response.message {
            println!("Got {:?} message:", message);
        }
    } else {
        println!("Error message: {:?}", response);
    }
    Ok(())
}

fn main() {
    // Add hosts to query:
    let hosts = [
        //"192.168.2.147:9999", // Clab-room-light
        "192.168.2.146:9999", // SC6800-NSLEDS1
        "192.168.2.101:9999", // SC6800-NSLEDS3
    ];

    // Populate plugs
    let mut entry: HashMap<String, bool> = HashMap::new();
    let mut plugs: Vec<SmartPlug> = Vec::new();
    for host in hosts.iter() {
        plugs.push(SmartPlug::new(host));
    }

    // Loop forever
    loop {
        // Loop each plug
        let mut i = 0;
        for plug in plugs.iter() {
            // Query plug info
            let info = plug.sysinfo();
            let info = match info {
                Ok(info) => info,
                Err(_err) => {
                    println!("Host: {}, Error getting plug info!", hosts[i]);
                    continue;
                }
            };

            // Retrieve info from the plug.
            let (mac, alias, relay_state) = match info.system {
                Some(sys) => (sys.get_sysinfo.mac, sys.get_sysinfo.alias, sys.get_sysinfo.relay_state != 0),
                None => {
                    println!("Host: {}, Error decoding plug info!", hosts[i]);
                    continue;
                }
            };

            // Insert current state into map and return old state.
            let old_val = entry.insert(mac.clone(), relay_state).unwrap_or(relay_state);

            // If state change send message.
            let changed = if old_val != relay_state {
                // Populate info for printing.
                let (state, icon) = if relay_state == true {
                    ("ON", ":red_circle:")
                } else {
                    ("OFF", ":large_blue_circle:")
                };

                // Format string.
                let msg = format!("{}{} *{}* switched *{}*! {}{}", icon, icon, &alias, state, icon, icon);

                // Send message.
                if let Err(err) = send_slack_message(&msg) {
                    println!("{}", err);
                }

                true // return changed.
            } else {
                false // return unchanged.
            };

            // Print device state
            println!(
                "Host: {}, MAC: {}, Device: {}, Relay State: {}, Changed {}",
                hosts[i], mac, alias, relay_state, changed
            );

            // Sleep before querying again.
            i += 1;
        }
        thread::sleep(Duration::from_secs(30));
        println!("========");
    }
}
