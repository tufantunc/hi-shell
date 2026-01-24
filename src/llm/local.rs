use crate::config::{Config, LocalProviderType};
use crate::error::{HiShellError, Result};
use crate::llm::{CommandResponse, LlmBackend, Message};
use async_trait::async_trait;
use serde_json::json;
use tracing::{debug, info};

pub struct LocalClient {
    config: Config,
}

impl LocalClient {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn list_models(provider: &LocalProviderType, base_url: &str) -> Result<Vec<String>> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        match provider {
            LocalProviderType::Ollama => {
                let url = if base_url.ends_with("/api/generate") {
                    base_url.replace("/api/generate", "/api/tags")
                } else if base_url.ends_with('/') {
                    format!("{}api/tags", base_url)
                } else {
                    format!("{}/api/tags", base_url)
                };

                let res = client.get(url).send().await?;
                if !res.status().is_success() {
                    return Err(HiShellError::Api {
                        provider: "Ollama".to_string(),
                        message: format!("Failed to list models: {}", res.status()),
                    });
                }

                let json: serde_json::Value = res.json().await?;
                let models = json["models"]
                    .as_array()
                    .ok_or_else(|| {
                        HiShellError::Parsing("Unexpected response from Ollama".to_string())
                    })?
                    .iter()
                    .filter_map(|m| m["name"].as_str().map(|s| s.to_string()))
                    .collect();
                Ok(models)
            }
            LocalProviderType::LmStudio => {
                let url = if base_url.ends_with("/v1/chat/completions") {
                    base_url.replace("/v1/chat/completions", "/v1/models")
                } else if base_url.ends_with('/') {
                    format!("{}v1/models", base_url)
                } else {
                    format!("{}/v1/models", base_url)
                };

                let res = client.get(url).send().await?;
                if !res.status().is_success() {
                    return Err(HiShellError::Api {
                        provider: "LM Studio".to_string(),
                        message: format!("Failed to list models: {}", res.status()),
                    });
                }

                let json: serde_json::Value = res.json().await?;
                let models = json["data"]
                    .as_array()
                    .ok_or_else(|| {
                        HiShellError::Parsing("Unexpected response from LM Studio".to_string())
                    })?
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
    async fn generate_command(
        &self,
        messages: &[Message],
        repair_context: Option<&str>,
    ) -> Result<CommandResponse> {
        let provider = self
            .config
            .local_provider
            .as_ref()
            .ok_or_else(|| HiShellError::Config("Local provider not configured".to_string()))?;

        let url = self
            .config
            .local_url
            .as_deref()
            .unwrap_or("http://localhost:11434/api/generate");

        let model = self.config.local_model.as_deref().unwrap_or("phi3");

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;
        let system_prompt = crate::llm::get_system_prompt(repair_context);

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

                debug!("Sending request to Ollama: {}", url);
                let res = client.post(url).json(&body).send().await?;

                if !res.status().is_success() {
                    return Err(HiShellError::Api {
                        provider: "Ollama".to_string(),
                        message: res.text().await?,
                    });
                }

                let json: serde_json::Value = res.json().await?;
                json["response"]
                    .as_str()
                    .ok_or_else(|| {
                        HiShellError::Parsing("Failed to parse response from Ollama".to_string())
                    })?
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

                debug!("Sending request to LM Studio: {}", url);
                let res = client.post(url).json(&body).send().await?;

                if !res.status().is_success() {
                    return Err(HiShellError::Api {
                        provider: "LM Studio".to_string(),
                        message: res.text().await?,
                    });
                }

                let json: serde_json::Value = res.json().await?;
                json["choices"][0]["message"]["content"]
                    .as_str()
                    .ok_or_else(|| {
                        HiShellError::Parsing("Failed to parse response from LM Studio".to_string())
                    })?
                    .to_string()
            }
        };

        info!("Command generated successfully using local {:?}", provider);
        crate::llm::parse_llm_response(&res_text)
    }
}
