use slack_api::sync as client;

// Send slack message
pub fn send_message(channel: &str, text: &str) -> Result<(), Box<dyn std::error::Error>> {
    let token = "***REMOVED***";
    let client = client::default_client().map_err(|_| "Could not get default_client")?;

    client::chat::post_message(
        &client,
        &token,
        &client::chat::PostMessageRequest {
            channel: channel,
            text: text,
            username: Some("smartplug_notifier"),
            ..client::chat::PostMessageRequest::default()
        },
    )?;

    Ok(())
}
