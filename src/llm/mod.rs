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
