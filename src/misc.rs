use std::error::Error;

pub fn download_text(url: &str) -> Result<String, Box<dyn Error>> {
    let dur = std::time::Duration::from_secs(5);

    let agent: ureq::Agent = ureq::AgentBuilder::new()
        .timeout_read(dur)
        .timeout_write(dur)
        .build();

    let text = agent.get(url).call()?.into_string()?;
    Ok(text)
}
