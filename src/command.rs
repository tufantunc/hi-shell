use anyhow::Result;

pub fn execute_command(cmd: &str) -> Result<(String, bool)> {
    #[cfg(windows)]
    let output = std::process::Command::new("cmd")
        .args(["/C", cmd])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?
        .wait_with_output()?;

    #[cfg(not(windows))]
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?
        .wait_with_output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let mut combined = stdout.clone();
    if !stderr.is_empty() {
        combined.push_str("\nError:\n");
        combined.push_str(&stderr);
    }

    Ok((combined, output.status.success()))
}

pub fn truncate_output(output: &str, max_len: usize) -> String {
    if output.len() <= max_len {
        return output.to_string();
    }

    let boundary = output
        .char_indices()
        .take_while(|(i, _)| *i < max_len)
        .last()
        .map(|(i, c)| i + c.len_utf8())
        .unwrap_or(max_len);

    format!("{}... (truncated)", &output[..boundary])
}
