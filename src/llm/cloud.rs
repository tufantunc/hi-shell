use crate::config::{CloudProviderType, Config};
use crate::error::{HiShellError, Result};
use crate::llm::{CommandResponse, LlmBackend, Message};
use async_trait::async_trait;
use serde_json::json;
use tracing::{debug, info};

pub struct CloudClient {
    config: Config,
    base_url_override: Option<String>,
}

impl CloudClient {
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

    pub async fn list_models(
        provider: &CloudProviderType,
        api_key: Option<&str>,
    ) -> Result<Vec<String>> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        match provider {
            CloudProviderType::OpenRouter => {
                let url = "https://openrouter.ai/api/v1/models";
                let res = client.get(url).send().await?;
                if !res.status().is_success() {
                    return Err(HiShellError::Api {
                        provider: "OpenRouter".to_string(),
                        message: format!("Failed to fetch models: {}", res.status()),
                    });
                }
                let json: serde_json::Value = res.json().await?;
                let models = json["data"]
                    .as_array()
                    .ok_or_else(|| {
                        HiShellError::Parsing("Unexpected response from OpenRouter".to_string())
                    })?
                    .iter()
                    .filter_map(|m| m["id"].as_str().map(|s| s.to_string()))
                    .collect();
                Ok(models)
            }
            CloudProviderType::Gemini => {
                let api_key = api_key.ok_or_else(|| {
                    HiShellError::Config("API key required to list Gemini models".to_string())
                })?;
                let url = format!(
                    "https://generativelanguage.googleapis.com/v1beta/models?key={}",
                    api_key
                );
                let res = client.get(url).send().await?;
                if !res.status().is_success() {
                    return Err(HiShellError::Api {
                        provider: "Gemini".to_string(),
                        message: format!("Failed to fetch models: {}", res.status()),
                    });
                }
                let json: serde_json::Value = res.json().await?;
                let models = json["models"]
                    .as_array()
                    .ok_or_else(|| {
                        HiShellError::Parsing("Unexpected response from Gemini".to_string())
                    })?
                    .iter()
                    .filter_map(|m| m["name"].as_str().map(|s| s.replace("models/", "")))
                    .collect();
                Ok(models)
            }
            CloudProviderType::Anthropic => {
                let api_key = api_key.ok_or_else(|| {
                    HiShellError::Config("API key required to list Anthropic models".to_string())
                })?;
                let url = "https://api.anthropic.com/v1/models";
                let res = client
                    .get(url)
                    .header("x-api-key", api_key)
                    .header("anthropic-version", "2023-06-01")
                    .send()
                    .await?;
                if !res.status().is_success() {
                    return Err(HiShellError::Api {
                        provider: "Anthropic".to_string(),
                        message: format!("Failed to fetch models: {}", res.status()),
                    });
                }
                let json: serde_json::Value = res.json().await?;
                let models = json["data"]
                    .as_array()
                    .ok_or_else(|| {
                        HiShellError::Parsing("Unexpected response from Anthropic".to_string())
                    })?
                    .iter()
                    .filter_map(|m| m["id"].as_str().map(|s| s.to_string()))
                    .collect();
                Ok(models)
            }
            CloudProviderType::OpenAI => {
                let api_key = api_key.ok_or_else(|| {
                    HiShellError::Config("API key required to list OpenAI models".to_string())
                })?;
                let url = "https://api.openai.com/v1/models";
                let res = client
                    .get(url)
                    .header("Authorization", format!("Bearer {}", api_key))
                    .send()
                    .await?;
                if !res.status().is_success() {
                    return Err(HiShellError::Api {
                        provider: "OpenAI".to_string(),
                        message: format!("Failed to fetch models: {}", res.status()),
                    });
                }
                let json: serde_json::Value = res.json().await?;
                let models = json["data"]
                    .as_array()
                    .ok_or_else(|| {
                        HiShellError::Parsing("Unexpected response from OpenAI".to_string())
                    })?
                    .iter()
                    .filter_map(|m| m["id"].as_str().map(|s| s.to_string()))
                    .collect();
                Ok(models)
            }
            CloudProviderType::Custom => Ok(vec![]),
        }
    }
}

#[async_trait]
impl LlmBackend for CloudClient {
    async fn generate_command(
        &self,
        messages: &[Message],
        repair_context: Option<&str>,
    ) -> Result<CommandResponse> {
        let provider = self
            .config
            .cloud_provider
            .as_ref()
            .ok_or_else(|| HiShellError::Config("Cloud provider not configured".to_string()))?;

        let api_key = self
            .config
            .api_key
            .as_ref()
            .ok_or_else(|| HiShellError::Config("API key not configured".to_string()))?;

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()?;
        let system_prompt = crate::llm::get_system_prompt(repair_context);

        let pb = crate::llm::create_spinner(if repair_context.is_some() {
            "Analyzing error and fixing..."
        } else {
            "Generating command..."
        })?;

        let (url, body) = match provider {
            CloudProviderType::OpenRouter => {
                let url = self
                    .base_url_override
                    .as_deref()
                    .unwrap_or("https://openrouter.ai/api/v1/chat/completions");
                let model = self
                    .config
                    .cloud_model
                    .as_deref()
                    .unwrap_or("google/gemini-2.0-flash-exp");

                let mut api_messages = vec![json!({"role": "system", "content": system_prompt})];
                for msg in messages {
                    let role = match msg.role {
                        crate::llm::Role::System => "user",
                        crate::llm::Role::User => "user",
                        crate::llm::Role::Assistant => "assistant",
                    };
                    api_messages.push(json!({"role": role, "content": msg.content}));
                }

                let body = json!({
                    "model": model,
                    "messages": api_messages
                });
                (url.to_string(), body)
            }
            CloudProviderType::Gemini => {
                let model = self
                    .config
                    .cloud_model
                    .as_deref()
                    .unwrap_or("gemini-1.5-flash");
                let url = if let Some(ref base) = self.base_url_override {
                    base.clone()
                } else {
                    format!(
                        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
                        model, api_key
                    )
                };

                let mut combined_prompt = format!("{}\n\nConversation History:\n", system_prompt);
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
                    "contents": [{
                        "parts": [{
                            "text": combined_prompt
                        }]
                    }],
                    "generationConfig": {
                        "responseMimeType": "application/json"
                    }
                });
                (url, body)
            }
            CloudProviderType::Anthropic => {
                let url = self
                    .base_url_override
                    .as_deref()
                    .unwrap_or("https://api.anthropic.com/v1/messages");
                let model = self
                    .config
                    .cloud_model
                    .as_deref()
                    .unwrap_or("claude-3-5-sonnet-20240620");

                let mut api_messages = Vec::new();
                for msg in messages {
                    let role = match msg.role {
                        crate::llm::Role::Assistant => "assistant",
                        _ => "user",
                    };
                    api_messages.push(json!({"role": role, "content": msg.content}));
                }

                let body = json!({
                    "model": model,
                    "max_tokens": 1024,
                    "system": system_prompt,
                    "messages": api_messages
                });
                (url.to_string(), body)
            }
            CloudProviderType::OpenAI => {
                let url = self
                    .base_url_override
                    .as_deref()
                    .unwrap_or("https://api.openai.com/v1/chat/completions");
                let model = self.config.cloud_model.as_deref().unwrap_or("gpt-4o");

                let mut api_messages = vec![json!({"role": "system", "content": system_prompt})];
                for msg in messages {
                    let role = match msg.role {
                        crate::llm::Role::System => "user",
                        crate::llm::Role::User => "user",
                        crate::llm::Role::Assistant => "assistant",
                    };
                    api_messages.push(json!({"role": role, "content": msg.content}));
                }

                let body = json!({
                    "model": model,
                    "messages": api_messages
                });
                (url.to_string(), body)
            }
            CloudProviderType::Custom => {
                let base_url = self.config.cloud_custom_url.as_ref().ok_or_else(|| {
                    HiShellError::Config("Custom cloud URL not configured".to_string())
                })?;

                let url = if base_url.ends_with('/') {
                    format!("{}chat/completions", base_url)
                } else {
                    format!("{}/chat/completions", base_url)
                };
                let model = self.config.cloud_model.as_deref().ok_or_else(|| {
                    HiShellError::Config("Cloud model not configured".to_string())
                })?;

                let mut api_messages = vec![json!({"role": "system", "content": system_prompt})];
                for msg in messages {
                    let role = match msg.role {
                        crate::llm::Role::System => "user",
                        crate::llm::Role::User => "user",
                        crate::llm::Role::Assistant => "assistant",
                    };
                    api_messages.push(json!({"role": role, "content": msg.content}));
                }

                let body = json!({
                    "model": model,
                    "messages": api_messages
                });
                (url, body)
            }
        };

        debug!("Sending request to {} with body: {:?}", url, body);
        let res = client.post(&url);
        let res = if *provider == CloudProviderType::OpenRouter
            || *provider == CloudProviderType::Custom
            || *provider == CloudProviderType::OpenAI
        {
            res.header("Authorization", format!("Bearer {}", api_key))
        } else if *provider == CloudProviderType::Anthropic {
            res.header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
        } else {
            res
        };

        let res = res.json(&body).send().await?;

        if !res.status().is_success() {
            let error_text = res.text().await?;
            return Err(HiShellError::Api {
                provider: format!("{:?}", provider),
                message: error_text,
            });
        }

        let json: serde_json::Value = res.json().await?;
        debug!("Received JSON response: {:?}", json);

        let content = match provider {
            CloudProviderType::OpenRouter => json["choices"][0]["message"]["content"]
                .as_str()
                .ok_or_else(|| {
                    HiShellError::Parsing(format!("Failed to parse OpenRouter response: {}", json))
                })?
                .to_string(),
            CloudProviderType::Gemini => json["candidates"][0]["content"]["parts"][0]["text"]
                .as_str()
                .ok_or_else(|| {
                    HiShellError::Parsing("Failed to parse Gemini response".to_string())
                })?
                .to_string(),
            CloudProviderType::Anthropic => json["content"][0]["text"]
                .as_str()
                .ok_or_else(|| {
                    HiShellError::Parsing("Failed to parse Anthropic response".to_string())
                })?
                .to_string(),
            CloudProviderType::OpenAI | CloudProviderType::Custom => json["choices"][0]["message"]
                ["content"]
                .as_str()
                .ok_or_else(|| {
                    HiShellError::Parsing(format!("Failed to parse response: {}", json))
                })?
                .to_string(),
        };

        pb.finish_and_clear();
        info!("Command generated successfully using {:?}", provider);
        crate::llm::parse_llm_response(&content)
    }
}
