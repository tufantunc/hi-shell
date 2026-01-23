use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;

pub mod cloud;
pub mod embedded;
pub mod local;

#[derive(Debug, Deserialize)]
pub struct CommandResponse {
    pub command: String,
    pub explanation: Option<String>,
    pub dangerous: bool,
}

#[async_trait]
pub trait LlmBackend: Send + Sync {
    async fn generate_command(&self, user_request: &str) -> Result<CommandResponse>;
}

pub fn get_system_info() -> String {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "unknown".to_string());
    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string());

    format!(
        "Operating System: {} ({})\nShell: {}\nCurrent Working Directory: {}",
        os, arch, shell, cwd
    )
}

pub fn get_system_prompt() -> String {
    let system_info = get_system_info();
    format!(
        r#"You are a terminal command generator. You must response with a valid JSON object only. No markdown formatting.

COMPATIBILITY RULES:
1. Strictly follow the provided Operating System and Shell context.
2. Only suggest commands and flags that are supported on the detected platform.
3. On macOS (Darwin/BSD), AVOID GNU-only flags (e.g., use 'du -d 1' instead of 'du --max-depth=1').
4. On Windows, ensure syntax is correct for the detected shell (PowerShell or CMD).
5. If multiple ways exist, prioritize the most portable and standard version for the specific environment.

Context:
{}

Schema:
{{
  "command": "formatted bash/zsh/shell command",
  "explanation": "concise explanation",
  "dangerous": boolean (true if destructive like rm, dd, mkfs, or system modification, else false)
}}"#,
        system_info
    )
}
