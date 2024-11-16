pub fn send_notification(topic: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    ureq::post(&format!("https://ntfy.sh/{topic}"))
        .set("Content-Type", "text/plain")
        .send_string(message)?;
    Ok(())
}
