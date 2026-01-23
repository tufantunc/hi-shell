use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum LlmProvider {
    Embedded,
    Local,
    Cloud,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum LocalProviderType {
    Ollama,
    LmStudio,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum CloudProviderType {
    OpenRouter,
    Gemini,
    Anthropic,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct Config {
    pub installation_id: String,
    pub llm_provider: LlmProvider,
    pub embedded_model: Option<String>,
    pub embedded_model_file: Option<String>,
    pub local_provider: Option<LocalProviderType>,
    pub local_url: Option<String>,
    pub local_model: Option<String>,
    pub cloud_provider: Option<CloudProviderType>,
    pub cloud_model: Option<String>,
    pub api_key: Option<String>,
    pub telemetry_enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            installation_id: uuid::Uuid::new_v4().to_string(),
            llm_provider: LlmProvider::Embedded,
            embedded_model: Some("microsoft/Phi-3-mini-4k-instruct-gguf".to_string()),
            embedded_model_file: Some("Phi-3-mini-4k-instruct-q4.gguf".to_string()),
            local_provider: None,
            local_url: None,
            local_model: None,
            cloud_provider: None,
            cloud_model: Some("microsoft/phi-3-mini-4k-instruct".to_string()),
            api_key: None,
            telemetry_enabled: false,
        }
    }
}

impl Config {
    pub fn get_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "hi-shell", "hi-shell")
            .context("Could not determine config directory")?;
        let config_dir = proj_dirs.config_dir();
        fs::create_dir_all(config_dir)?;
        Ok(config_dir.join("config.toml"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::get_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::get_path()?;
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}
