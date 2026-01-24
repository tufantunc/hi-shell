use thiserror::Error;

#[derive(Error, Debug)]
pub enum HiShellError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("TOML error: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("API error ({provider}): {message}")]
    Api { provider: String, message: String },

    #[error("Parsing error: {0}")]
    Parsing(String),

    #[error("LLM loading error: {0}")]
    LlmLoad(String),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, HiShellError>;
