use hi_shell::config::{CloudProviderType, Config, LlmProvider};
use hi_shell::llm::cloud::CloudClient;
use hi_shell::llm::{LlmBackend, Message, Role};
use serde_json::{json, Value};
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

fn make_cloud_config(provider: CloudProviderType) -> Config {
    let mut config = Config::default();
    config.llm_provider = LlmProvider::Cloud;
    config.cloud_provider = Some(provider);
    config.api_key = Some("test-api-key".to_string());
    config.cloud_model = Some("test-model".to_string());
    config
}

fn make_openai_response(command: &str, explanation: &str, dangerous: bool) -> Value {
    let content = serde_json::to_string(&json!({
        "command": command,
        "explanation": explanation,
        "dangerous": dangerous
    }))
    .unwrap();
    json!({
        "choices": [{"message": {"content": content}}]
    })
}

fn make_anthropic_response(command: &str, explanation: &str, dangerous: bool) -> Value {
    let content = serde_json::to_string(&json!({
        "command": command,
        "explanation": explanation,
        "dangerous": dangerous
    }))
    .unwrap();
    json!({
        "content": [{"text": content}]
    })
}

fn make_gemini_response(command: &str, explanation: &str, dangerous: bool) -> Value {
    let content = serde_json::to_string(&json!({
        "command": command,
        "explanation": explanation,
        "dangerous": dangerous
    }))
    .unwrap();
    json!({
        "candidates": [{"content": {"parts": [{"text": content}]}}]
    })
}

#[tokio::test]
async fn test_openrouter_request_and_response() {
    let server = MockServer::start().await;
    let config = make_cloud_config(CloudProviderType::OpenRouter);
    let client = CloudClient::with_base_url(config, format!("{}/chat/completions", server.uri()));

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            make_openai_response("ls -la", "list files", false),
        ))
        .mount(&server)
        .await;

    let messages = vec![Message {
        role: Role::User,
        content: "list all files".to_string(),
    }];

    let result = client.generate_command(&messages, None).await.unwrap();
    assert_eq!(result.command, "ls -la");
    assert!(!result.dangerous);
}

#[tokio::test]
async fn test_openrouter_api_error() {
    let server = MockServer::start().await;
    let config = make_cloud_config(CloudProviderType::OpenRouter);
    let client = CloudClient::with_base_url(config, format!("{}/chat/completions", server.uri()));

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .mount(&server)
        .await;

    let messages = vec![Message {
        role: Role::User,
        content: "test".to_string(),
    }];

    let result = client.generate_command(&messages, None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_anthropic_response_parsing() {
    let server = MockServer::start().await;
    let config = make_cloud_config(CloudProviderType::Anthropic);
    let client = CloudClient::with_base_url(config, server.uri());

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            make_anthropic_response("pwd", "print directory", false),
        ))
        .mount(&server)
        .await;

    let messages = vec![Message {
        role: Role::User,
        content: "where am I".to_string(),
    }];

    let result = client.generate_command(&messages, None).await.unwrap();
    assert_eq!(result.command, "pwd");
}

#[tokio::test]
async fn test_openai_response_parsing() {
    let server = MockServer::start().await;
    let config = make_cloud_config(CloudProviderType::OpenAI);
    let client = CloudClient::with_base_url(config, server.uri());

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            make_openai_response("echo hello", "say hello", false),
        ))
        .mount(&server)
        .await;

    let messages = vec![Message {
        role: Role::User,
        content: "say hello".to_string(),
    }];

    let result = client.generate_command(&messages, None).await.unwrap();
    assert_eq!(result.command, "echo hello");
}

#[tokio::test]
async fn test_custom_provider_url_construction() {
    let server = MockServer::start().await;
    let base = server.uri().to_string();
    let base_trimmed = base.trim_end_matches('/').to_string();

    let mut config = Config::default();
    config.llm_provider = LlmProvider::Cloud;
    config.cloud_provider = Some(CloudProviderType::Custom);
    config.cloud_custom_url = Some(format!("{}/v1", base_trimmed));
    config.api_key = Some("test-key".to_string());
    config.cloud_model = Some("test-model".to_string());

    let client = CloudClient::new(config);

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            make_openai_response("echo custom", "custom provider", false),
        ))
        .mount(&server)
        .await;

    let messages = vec![Message {
        role: Role::User,
        content: "test custom".to_string(),
    }];

    let result = client.generate_command(&messages, None).await.unwrap();
    assert_eq!(result.command, "echo custom");
}

#[tokio::test]
async fn test_custom_provider_trailing_slash() {
    let server = MockServer::start().await;
    let base = format!("{}/v1/", server.uri().to_string().trim_end_matches('/'));

    let mut config = Config::default();
    config.llm_provider = LlmProvider::Cloud;
    config.cloud_provider = Some(CloudProviderType::Custom);
    config.cloud_custom_url = Some(base);
    config.api_key = Some("test-key".to_string());
    config.cloud_model = Some("test-model".to_string());

    let client = CloudClient::new(config);

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            make_openai_response("echo slash", "trailing slash test", false),
        ))
        .mount(&server)
        .await;

    let messages = vec![Message {
        role: Role::User,
        content: "test slash".to_string(),
    }];

    let result = client.generate_command(&messages, None).await.unwrap();
    assert_eq!(result.command, "echo slash");
}

#[tokio::test]
async fn test_gemini_response_parsing() {
    let server = MockServer::start().await;
    let config = make_cloud_config(CloudProviderType::Gemini);
    let model = config.cloud_model.as_deref().unwrap();
    let api_key = config.api_key.as_deref().unwrap();
    let override_url = format!(
        "{}/{}:generateContent?key={}",
        server.uri(),
        model,
        api_key
    );
    let client = CloudClient::with_base_url(config, override_url);

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            make_gemini_response("find . -name '*.rs'", "find rust files", false),
        ))
        .mount(&server)
        .await;

    let messages = vec![Message {
        role: Role::User,
        content: "find rust files".to_string(),
    }];

    let result = client.generate_command(&messages, None).await.unwrap();
    assert!(result.command.contains("find"));
}

#[tokio::test]
async fn test_repair_context_passed() {
    let server = MockServer::start().await;
    let config = make_cloud_config(CloudProviderType::OpenAI);
    let client = CloudClient::with_base_url(config, server.uri());

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            make_openai_response("ls", "fixed", false),
        ))
        .mount(&server)
        .await;

    let messages = vec![Message {
        role: Role::User,
        content: "list files".to_string(),
    }];

    let result = client
        .generate_command(&messages, Some("No such file or directory"))
        .await
        .unwrap();
    assert_eq!(result.command, "ls");
}
