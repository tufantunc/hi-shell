use hi_shell::config::{CloudProviderType, Config, LlmProvider};

#[test]
fn test_default_config() {
    let config = Config::default();
    assert_eq!(config.llm_provider, LlmProvider::Embedded);
    assert!(config.embedded_model.is_some());
    assert!(!config.telemetry_enabled);
}

#[test]
fn test_config_serialization() {
    let mut config = Config::default();
    config.api_key = Some("test-key".to_string());
    config.llm_provider = LlmProvider::Cloud;
    config.cloud_provider = Some(CloudProviderType::OpenAI);

    let toml_str = toml::to_string(&config).unwrap();
    assert!(toml_str.contains("api_key = \"test-key\""));
    assert!(toml_str.contains("llm_provider = \"Cloud\""));
    assert!(toml_str.contains("cloud_provider = \"OpenAI\""));

    let deserialized: Config = toml::from_str(&toml_str).unwrap();
    assert_eq!(deserialized.api_key.unwrap(), "test-key");
    assert_eq!(deserialized.llm_provider, LlmProvider::Cloud);
}
