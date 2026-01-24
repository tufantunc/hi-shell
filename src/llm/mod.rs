use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod cloud;
pub mod embedded;
pub mod local;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommandResponse {
    pub command: String,
    pub explanation: Option<String>,
    pub dangerous: bool,
}

#[async_trait]
pub trait LlmBackend: Send + Sync {
    async fn generate_command(
        &self,
        messages: &[Message],
        repair_context: Option<&str>,
    ) -> Result<CommandResponse>;
}

pub fn get_system_info() -> String {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "unknown".to_string());
    let cwd = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string());

    let mut files_list = String::new();
    if let Ok(entries) = std::fs::read_dir(&cwd) {
        let files: Vec<String> = entries
            .filter_map(|e| e.ok())
            .map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                if e.path().is_dir() {
                    format!("{}/", name)
                } else {
                    name
                }
            })
            .take(50) // Limit to avoid token bloat
            .collect();
        files_list = files.join(", ");
    }

    format!(
        "Operating System: {} ({})\nShell: {}\nCurrent Working Directory: {}\nFiles in CWD: {}",
        os, arch, shell, cwd, files_list
    )
}

pub fn get_system_prompt(repair_context: Option<&str>) -> String {
    let system_info = get_system_info();
    let mut prompt = format!(
        r#"You are a terminal command generator. You must response with a valid JSON object only. No markdown formatting.

COMPATIBILITY RULES:
1. Strictly follow the provided Operating System and Shell context.
2. Only suggest commands and flags that are supported on the detected platform.
3. On macOS (Darwin/BSD), AVOID GNU-only flags (e.g., use 'du -d 1' instead of 'du --max-depth=1').
4. On Windows, ensure syntax is correct for the detected shell (PowerShell or CMD).
5. If multiple ways exist, prioritize the most portable and standard version for the specific environment.
6. MANDATORY: You must escape backslashes in the "command" string (e.g., "C:\\Users" instead of "C:\Users").

Context:
{}

Schema:
{{
  "command": "formatted bash/zsh/shell command",
  "explanation": "concise explanation",
  "dangerous": boolean (true if destructive like rm, dd, mkfs, or system modification, else false)
}}"#,
        system_info
    );

    if let Some(repair) = repair_context {
        prompt.push_str(&format!(
            r#"

IMPORTANT: The previous command failed with this error: "{}"

STRICT REPAIR RULES:
1. DO NOT suggest the same failing command again.
2. Analyze the error carefully. If it's a "No such file" error:
   - Check the "Files in CWD" list above for similar names or typos.
   - Suggest a command using the actual existing file name if you find a match.
3. If it's a "command not found" error, suggest how to install it.
4. If you are unsure, suggest a command to help diagnose the issue (e.g., list files, check permissions) but explain this clearly."#,
            repair
        ));
    }

    prompt
}

/// Helper to clean and parse LLM JSON responses that might be slightly malformed.
pub fn parse_llm_response(content: &str) -> Result<CommandResponse> {
    // 1. Remove markdown code blocks
    let clean = content.replace("```json", "").replace("```", "");
    let clean = clean.trim();

    // 2. Extract content between first { and last }
    let json_str = if let (Some(start), Some(end)) = (clean.find('{'), clean.rfind('}')) {
        &clean[start..=end]
    } else {
        clean
    };

    // 3. Attempt to fix common escape errors (e.g. unescaped backslashes in paths)
    // This is a naive fix: replace single backslashes not followed by valid escape chars
    // However, it's safer to just try parsing first.
    match serde_json::from_str::<CommandResponse>(json_str) {
        Ok(res) => Ok(res),
        Err(e) => {
            // If it fails with an escape error, try a simple regex-based escaping of backslashes
            // only if they are not already part of a valid escape sequence.
            // For now, let's just return the error but with better context.
            Err(anyhow::anyhow!(
                "JSON Parse Error: {}. Raw content: {}",
                e,
                json_str
            ))
        }
    }
}
