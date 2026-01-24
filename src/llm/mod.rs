use crate::error::{HiShellError, Result};
use async_trait::async_trait;
use indicatif::{ProgressBar, ProgressStyle};
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

    // 2. Extract content of the FIRST JSON object found
    let json_str = if let Some(start) = clean.find('{') {
        // Find the matching closing brace using brace counting
        let mut depth = 0;
        let mut end_pos = start;
        for (i, c) in clean[start..].char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end_pos = start + i;
                        break;
                    }
                }
                _ => {}
            }
        }
        &clean[start..=end_pos]
    } else {
        clean
    };

    // 3. Try parsing directly first
    if let Ok(res) = serde_json::from_str::<CommandResponse>(json_str) {
        return Ok(res);
    }

    // 4. Try to extract command field directly with regex-like parsing
    if let Some(cmd) = extract_field_value(json_str, "command") {
        let explanation = extract_field_value(json_str, "explanation");
        let is_dangerous = extract_field_value(json_str, "is_dangerous")
            .map(|s| s.to_lowercase() == "true")
            .unwrap_or(false);

        return Ok(CommandResponse {
            command: cmd,
            explanation,
            dangerous: is_dangerous,
        });
    }

    // 5. Fallback: try to find any quoted string that looks like a command
    if let Some(cmd) = extract_field_value(json_str, "answer") {
        return Ok(CommandResponse {
            command: cmd,
            explanation: None,
            dangerous: false,
        });
    }

    Err(HiShellError::Parsing(format!(
        "Could not parse LLM response. Raw content: {}",
        json_str
    )))
}

/// Creates a styled spinner progress bar for LLM operations
pub fn create_spinner(message: &'static str) -> Result<ProgressBar> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.blue} {msg}")
            .map_err(|e| HiShellError::Config(e.to_string()))?,
    );
    pb.set_message(message);
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    Ok(pb)
}

fn extract_field_value(json_str: &str, field: &str) -> Option<String> {
    // Look for "field": "value" or "field" : "value" patterns
    let search_pattern = format!(r#""{}""#, field);

    if let Some(key_pos) = json_str.find(&search_pattern) {
        let after_key = &json_str[key_pos + search_pattern.len()..];

        // Skip whitespace and colon
        let after_colon = after_key.trim_start();
        if !after_colon.starts_with(':') {
            return None;
        }
        let after_colon = after_colon[1..].trim_start();

        // Check if value starts with quote
        if !after_colon.starts_with('"') {
            // Handle boolean/number values
            let end = after_colon
                .find(|c: char| c == ',' || c == '}' || c == '\n')
                .unwrap_or(after_colon.len());
            let value = after_colon[..end].trim();
            if !value.is_empty() {
                return Some(value.to_string());
            }
            return None;
        }

        // Extract quoted string value
        let value_start = 1; // Skip opening quote
        let remaining = &after_colon[value_start..];

        let mut chars = remaining.chars();
        let mut value = String::new();
        let mut escaped = false;

        for c in chars.by_ref() {
            if escaped {
                value.push(c);
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                break;
            } else {
                value.push(c);
            }
        }

        if !value.is_empty() {
            return Some(value);
        }
    }
    None
}
