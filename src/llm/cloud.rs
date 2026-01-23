use crate::config::{CloudProviderType, Config};
use crate::llm::{CommandResponse, LlmBackend, Message};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use serde_json::json;

pub struct CloudClient {
    config: Config,
}

impl CloudClient {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn list_models(
        provider: &CloudProviderType,
        api_key: Option<&str>,
    ) -> Result<Vec<String>> {
        let client = reqwest::Client::new();
        match provider {
            CloudProviderType::OpenRouter => {
                let url = "https://openrouter.ai/api/v1/models";
                let res = client.get(url).send().await?;
                if !res.status().is_success() {
                    return Err(anyhow!("Failed to fetch OpenRouter models"));
                }
                let json: serde_json::Value = res.json().await?;
                let models = json["data"]
                    .as_array()
                    .ok_or_else(|| anyhow!("Unexpected response from OpenRouter"))?
                    .iter()
                    .filter_map(|m| m["id"].as_str().map(|s| s.to_string()))
                    .collect();
                Ok(models)
            }
            CloudProviderType::Gemini => {
                let api_key =
                    api_key.ok_or_else(|| anyhow!("API key required to list Gemini models"))?;
                let url = format!(
                    "https://generativelanguage.googleapis.com/v1beta/models?key={}",
                    api_key
                );
                let res = client.get(url).send().await?;
                if !res.status().is_success() {
                    return Err(anyhow!("Failed to fetch Gemini models"));
                }
                let json: serde_json::Value = res.json().await?;
                let models = json["models"]
                    .as_array()
                    .ok_or_else(|| anyhow!("Unexpected response from Gemini"))?
                    .iter()
                    .filter_map(|m| {
                        m["name"].as_str().map(|s| {
                            // Gemini model names are usually 'models/gemini-1.5-flash'
                            s.replace("models/", "")
                        })
                    })
                    .collect();
                Ok(models)
            }
            CloudProviderType::Anthropic => {
                let api_key =
                    api_key.ok_or_else(|| anyhow!("API key required to list Anthropic models"))?;
                let url = "https://api.anthropic.com/v1/models";
                let res = client
                    .get(url)
                    .header("x-api-key", api_key)
                    .header("anthropic-version", "2023-06-01")
                    .send()
                    .await?;
                if !res.status().is_success() {
                    return Err(anyhow!("Failed to fetch Anthropic models"));
                }
                let json: serde_json::Value = res.json().await?;
                let models = json["data"]
                    .as_array()
                    .ok_or_else(|| anyhow!("Unexpected response from Anthropic"))?
                    .iter()
                    .filter_map(|m| m["id"].as_str().map(|s| s.to_string()))
                    .collect();
                Ok(models)
            }
        }
    }
}

#[async_trait]
impl LlmBackend for CloudClient {
    async fn generate_command(&self, messages: &[Message]) -> Result<CommandResponse> {
        let provider = self
            .config
            .cloud_provider
            .as_ref()
            .ok_or_else(|| anyhow!("Cloud provider not configured"))?;

        let api_key = self
            .config
            .api_key
            .as_ref()
            .ok_or_else(|| anyhow!("API key not configured"))?;

        let client = reqwest::Client::new();
        let system_prompt = crate::llm::get_system_prompt();

        let (url, body) = match provider {
            CloudProviderType::OpenRouter => {
                let url = "https://openrouter.ai/api/v1/chat/completions";
                let model = self
                    .config
                    .cloud_model
                    .as_deref()
                    .unwrap_or("google/gemini-2.0-flash-exp");

                let mut api_messages = vec![json!({"role": "system", "content": system_prompt})];
                for msg in messages {
                    let role = match msg.role {
                        crate::llm::Role::System => "user", // OpenRouter/others might not like tool output as system
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
                let url = format!(
                    "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
                    model, api_key
                );

                // Gemini expects a different format, combining system prompt into the first message or instruction
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
                let url = "https://api.anthropic.com/v1/messages";
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
        };

        let mut request = client.post(&url);

        if *provider == CloudProviderType::OpenRouter {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        } else if *provider == CloudProviderType::Anthropic {
            request = request
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01");
        }

        let res = request.json(&body).send().await?;

        if !res.status().is_success() {
            let error_text = res.text().await?;
            return Err(anyhow!("API Error: {}", error_text));
        }

        let json: serde_json::Value = res.json().await?;

        let content = match provider {
            CloudProviderType::OpenRouter => json["choices"][0]["message"]["content"]
                .as_str()
                .ok_or_else(|| anyhow!("Failed to parse OpenRouter response"))?
                .to_string(),
            CloudProviderType::Gemini => json["candidates"][0]["content"]["parts"][0]["text"]
                .as_str()
                .ok_or_else(|| anyhow!("Failed to parse Gemini response"))?
                .to_string(),
            CloudProviderType::Anthropic => json["content"][0]["text"]
                .as_str()
                .ok_or_else(|| anyhow!("Failed to parse Anthropic response"))?
                .to_string(),
        };

        crate::llm::parse_llm_response(&content)
    }
}
