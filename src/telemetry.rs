use crate::config::Config;
use serde_json::json;

pub struct Telemetry {
    api_key: Option<&'static str>,
    installation_id: String,
    enabled: bool,
}

impl Telemetry {
    pub fn new(config: &Config) -> Self {
        // API Key is injected at build time via environment variable
        let api_key = option_env!("HI_SHELL_POSTHOG_KEY");

        Self {
            api_key,
            installation_id: config.installation_id.clone(),
            enabled: config.telemetry_enabled && api_key.is_some(),
        }
    }

    /// Tracks an event with metadata ONLY.
    /// STRICTLY NO USER CONTENT (commands, prompts, outputs) allowed.
    pub fn track_event(&self, event_name: &str, properties: serde_json::Value) {
        if !self.enabled {
            return;
        }

        let api_key = self.api_key.unwrap().to_string(); // Safe because enabled checks is_some
        let event_name = event_name.to_string();
        let distinct_id = self.installation_id.clone();

        // Merge event properties with system info
        let mut final_props = properties;
        if let Some(obj) = final_props.as_object_mut() {
            obj.insert("distinct_id".to_string(), json!(distinct_id));
            obj.insert("platform_os".to_string(), json!(std::env::consts::OS));
            obj.insert("platform_arch".to_string(), json!(std::env::consts::ARCH));
        }

        tokio::spawn(async move {
            let client = reqwest::Client::new();
            let body = json!({
                "api_key": api_key,
                "event": event_name,
                "properties": final_props,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            });

            let _ = client
                .post("https://app.posthog.com/capture/")
                .json(&body)
                .send()
                .await;
        });
    }
}
