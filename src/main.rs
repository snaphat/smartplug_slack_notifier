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

            // Retrieve alias name and relay state
            let alias;
            let relay_state;
            let mut changed = false;
            match info.system {
                Some(sys) => {
                    //dev_name = sys.get_sysinfo.dev_name;
                    alias = sys.get_sysinfo.alias;
                    relay_state = sys.get_sysinfo.relay_state != 0;
                    let old_val = entry.insert(alias.clone(), relay_state);
                    let old_val = old_val.unwrap_or(relay_state);
                    if old_val != relay_state {
                        changed = true;
                        let state = match relay_state {
                            true => "ON",
                            false => "OFF",
                        };
                        let icon = match relay_state {
                            true => ":red_circle:",
                            false => ":large_blue_circle:",
                        };
                        let msg = format!("{}{} *{}* switched *{}*! {}{}",icon, icon, alias, state, icon, icon);
                        match send_slack_message(&msg) {
                            Ok(_) => (),
                            Err(err) => println!("{}", err),
                        };
                    }
                }
                None => {
                    println!("Host: {}, Error decoding plug info!", hosts[i]);
                    continue;
                }
            };

            // Print device state
            println!(
                "Host: {}, Device: {}, Relay State: {}, Changed {}",
                hosts[i], alias, relay_state, changed
            );

            // Sleep before querying again.
            i += 1;
        }
        thread::sleep(Duration::from_secs(30));
        println!("========");
    }
}
