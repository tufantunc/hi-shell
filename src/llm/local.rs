use crate::config::{Config, LocalProviderType};
use crate::llm::{CommandResponse, LlmBackend, Message};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use serde_json::json;

pub struct LocalClient {
    config: Config,
}

impl LocalClient {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn list_models(provider: &LocalProviderType, base_url: &str) -> Result<Vec<String>> {
        let client = reqwest::Client::new();
        match provider {
            LocalProviderType::Ollama => {
                // Ollama's tags endpoint is usually at /api/tags
                // The URL in config is /api/generate usually, so we need to adjust
                let url = if base_url.ends_with("/api/generate") {
                    base_url.replace("/api/generate", "/api/tags")
                } else if base_url.ends_with("/") {
                    format!("{}api/tags", base_url)
                } else {
                    format!("{}/api/tags", base_url)
                };

                let res = client.get(url).send().await?;
                if !res.status().is_success() {
                    return Err(anyhow!("Failed to list Ollama models"));
                }

                let json: serde_json::Value = res.json().await?;
                let models = json["models"]
                    .as_array()
                    .ok_or_else(|| anyhow!("Unexpected response from Ollama"))?
                    .iter()
                    .filter_map(|m| m["name"].as_str().map(|s| s.to_string()))
                    .collect();
                Ok(models)
            }
            LocalProviderType::LmStudio => {
                // LM Studio usually uses OpenAI compatible /v1/models
                let url = if base_url.ends_with("/v1/chat/completions") {
                    base_url.replace("/v1/chat/completions", "/v1/models")
                } else if base_url.ends_with("/") {
                    format!("{}v1/models", base_url)
                } else {
                    format!("{}/v1/models", base_url)
                };

                let res = client.get(url).send().await?;
                if !res.status().is_success() {
                    return Err(anyhow!("Failed to list LM Studio models"));
                }

                let json: serde_json::Value = res.json().await?;
                let models = json["data"]
                    .as_array()
                    .ok_or_else(|| anyhow!("Unexpected response from LM Studio"))?
                    .iter()
                    .filter_map(|m| m["id"].as_str().map(|s| s.to_string()))
                    .collect();
                Ok(models)
            }
        }
    }
}

#[async_trait]
impl LlmBackend for LocalClient {
    async fn generate_command(&self, messages: &[Message]) -> Result<CommandResponse> {
        let provider = self
            .config
            .local_provider
            .as_ref()
            .ok_or_else(|| anyhow!("Local provider not configured"))?;

        let url = self
            .config
            .local_url
            .as_deref()
            .unwrap_or("http://localhost:11434/api/generate");

        let model = self.config.local_model.as_deref().unwrap_or("phi3");

        let client = reqwest::Client::new();
        let system_prompt = crate::llm::get_system_prompt();

        let res_text = match provider {
            LocalProviderType::Ollama => {
                let mut combined_prompt = format!("System: {}\n", system_prompt);
                for msg in messages {
                    let role_str = match msg.role {
                        crate::llm::Role::System => "System/Output",
                        crate::llm::Role::User => "User",
                        crate::llm::Role::Assistant => "Assistant",
                    };
                    combined_prompt.push_str(&format!("{}: {}\n", role_str, msg.content));
                }
                combined_prompt.push_str("\nGenerate the next command:");

                let body = json!({
                    "model": model,
                    "prompt": combined_prompt,
                    "stream": false,
                    "format": "json"
                });

                let res = client.post(url).json(&body).send().await?;

                if !res.status().is_success() {
                    let error_text = res.text().await?;
                    return Err(anyhow!("Ollama API Error: {}", error_text));
                }

                let json: serde_json::Value = res.json().await?;
                json["response"]
                    .as_str()
                    .ok_or_else(|| anyhow!("Failed to parse response from Ollama"))?
                    .to_string()
            }
            LocalProviderType::LmStudio => {
                let mut api_messages = vec![json!({"role": "system", "content": system_prompt})];
                for msg in messages {
                    let role = match msg.role {
                        crate::llm::Role::Assistant => "assistant",
                        _ => "user",
                    };
                    api_messages.push(json!({"role": role, "content": msg.content}));
                }

                let body = json!({
                    "model": model,
                    "messages": api_messages,
                    "response_format": { "type": "json_object" }
                });

                let res = client.post(url).json(&body).send().await?;

                if !res.status().is_success() {
                    let error_text = res.text().await?;
                    return Err(anyhow!("LM Studio API Error: {}", error_text));
                }

                let json: serde_json::Value = res.json().await?;
                json["choices"][0]["message"]["content"]
                    .as_str()
                    .ok_or_else(|| anyhow!("Failed to parse response from LM Studio"))?
                    .to_string()
            }
        };

        crate::llm::parse_llm_response(&res_text)
    }
}
