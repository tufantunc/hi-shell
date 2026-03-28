use hi_shell::config::{CloudProviderType, Config, LlmProvider, LocalProviderType};

#[test]
fn test_config_save_and_load_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");

    let mut config = Config::default();
    config.llm_provider = LlmProvider::Cloud;
    config.cloud_provider = Some(CloudProviderType::OpenRouter);
    config.api_key = Some("test-api-key-12345".to_string());
    config.cloud_model = Some("google/gemini-2.0-flash-exp".to_string());

    config.save_to(&path).unwrap();
    assert!(path.exists());

    let loaded = Config::load_from(&path).unwrap();
    assert_eq!(loaded.llm_provider, LlmProvider::Cloud);
    assert_eq!(loaded.cloud_provider, Some(CloudProviderType::OpenRouter));
    assert_eq!(loaded.api_key, Some("test-api-key-12345".to_string()));
    assert_eq!(
        loaded.cloud_model,
        Some("google/gemini-2.0-flash-exp".to_string())
    );
}

#[test]
fn test_config_load_missing_file_returns_default() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nonexistent.toml");

    let loaded = Config::load_from(&path).unwrap();
    assert_eq!(loaded.llm_provider, LlmProvider::Embedded);
    assert!(loaded.telemetry_enabled == false);
}

#[test]
fn test_config_load_corrupted_toml() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");

    std::fs::write(&path, "this is not valid toml [[[[").unwrap();
    let result = Config::load_from(&path);
    assert!(result.is_err());
}

#[test]
fn test_config_all_providers_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");

    let providers = vec![
        (LlmProvider::Embedded, None, None),
        (LlmProvider::Local, Some(LocalProviderType::Ollama), None),
        (LlmProvider::Cloud, None, Some(CloudProviderType::Anthropic)),
    ];

    for (provider, local_prov, cloud_prov) in providers {
        let mut config = Config::default();
        config.llm_provider = provider.clone();
        config.local_provider = local_prov.clone();
        config.cloud_provider = cloud_prov.clone();

        config.save_to(&path).unwrap();
        let loaded = Config::load_from(&path).unwrap();

        assert_eq!(loaded.llm_provider, provider);
        assert_eq!(loaded.local_provider, local_prov);
        assert_eq!(loaded.cloud_provider, cloud_prov);
    }
}

#[test]
fn test_config_default_values_with_partial_toml() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");

    std::fs::write(&path, "llm_provider = \"Cloud\"\n").unwrap();
    let loaded = Config::load_from(&path).unwrap();

    assert_eq!(loaded.llm_provider, LlmProvider::Cloud);
    assert!(loaded.api_key.is_none());
    assert!(loaded.telemetry_enabled == false);
}

#[test]
fn test_config_api_key_preserved() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");

    let mut config = Config::default();
    config.api_key = Some("sk-proj-abc123XYZ!@#".to_string());

    config.save_to(&path).unwrap();
    let loaded = Config::load_from(&path).unwrap();
    assert_eq!(loaded.api_key, Some("sk-proj-abc123XYZ!@#".to_string()));
}

#[test]
fn test_save_to_creates_parent_directories() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nested").join("deep").join("config.toml");

    let config = Config::default();
    config.save_to(&path).unwrap();
    assert!(path.exists());
}
