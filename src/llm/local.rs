use crate::config::{Config, LocalProviderType};
use crate::error::{HiShellError, Result};
use crate::llm::{CommandResponse, LlmBackend, Message};
use async_trait::async_trait;
use serde_json::json;
use tracing::{debug, info};

pub struct LocalClient {
    config: Config,
    base_url_override: Option<String>,
}

impl LocalClient {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            base_url_override: None,
        }
    }

    pub fn with_base_url(config: Config, base_url: String) -> Self {
        Self {
            config,
            base_url_override: Some(base_url),
        }
    }

    fn normalize_base_url(base_url: &str) -> String {
        base_url.trim_end_matches('/').to_string()
    }

    fn build_ollama_url(base_url: &str, endpoint: &str) -> String {
        let base = Self::normalize_base_url(base_url);
        format!("{}/api/{}", base, endpoint)
    }

    fn build_lmstudio_url(base_url: &str, endpoint: &str) -> String {
        let base = Self::normalize_base_url(base_url);
        format!("{}/v1/{}", base, endpoint)
    }

    pub async fn list_models(provider: &LocalProviderType, base_url: &str) -> Result<Vec<String>> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        match provider {
            LocalProviderType::Ollama => {
                let url = Self::build_ollama_url(base_url, "tags");

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
                let url = Self::build_lmstudio_url(base_url, "models");

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

        let effective_url = self
            .base_url_override
            .as_deref()
            .unwrap_or_else(|| {
                self.config
                    .local_url
                    .as_deref()
                    .unwrap_or("http://localhost:11434")
            });

        let model = self.config.local_model.as_deref().unwrap_or("phi3");

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;
        let system_prompt = crate::llm::get_system_prompt(repair_context);

        let pb = crate::llm::create_spinner(if repair_context.is_some() {
            "Analyzing error and fixing..."
        } else {
            "Generating command..."
        })?;

        let res_text = match provider {
            LocalProviderType::Ollama => {
                let url = Self::build_ollama_url(effective_url, "generate");
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
                let res = client.post(&url).json(&body).send().await?;

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
                let url = Self::build_lmstudio_url(effective_url, "chat/completions");
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
                    "response_format": {
                        "type": "json_schema",
                        "json_schema": {
                            "name": "command_response",
                            "strict": true,
                            "schema": {
                                "type": "object",
                                "properties": {
                                    "command": { "type": "string" },
                                    "explanation": { "type": "string" }
                                },
                                "required": ["command", "explanation"],
                                "additionalProperties": false
                            }
                        }
                    }
                });

                debug!("Sending request to LM Studio: {}", url);
                let res = client.post(&url).json(&body).send().await?;

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

        pb.finish_and_clear();
        info!("Command generated successfully using local {:?}", provider);
        crate::llm::parse_llm_response(&res_text)
    }
}
