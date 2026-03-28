use hi_shell::config::{Config, LlmProvider, LocalProviderType};
use hi_shell::llm::local::LocalClient;
use hi_shell::llm::{LlmBackend, Message, Role};
use serde_json::json;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

fn make_local_config(provider: LocalProviderType, base_url: &str) -> Config {
    let mut config = Config::default();
    config.llm_provider = LlmProvider::Local;
    config.local_provider = Some(provider);
    config.local_url = Some(base_url.to_string());
    config.local_model = Some("test-model".to_string());
    config
}

#[tokio::test]
async fn test_ollama_request_and_response() {
    let server = MockServer::start().await;
    let base_url = server.uri().to_string();
    let config = make_local_config(LocalProviderType::Ollama, &base_url);
    let client = LocalClient::new(config);

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "response": "{\"command\": \"ls -la\", \"explanation\": \"list files\", \"dangerous\": false}"
        })))
        .mount(&server)
        .await;

    let messages = vec![Message {
        role: Role::User,
        content: "list files".to_string(),
    }];

    let result = client.generate_command(&messages, None).await.unwrap();
    assert_eq!(result.command, "ls -la");
    assert!(!result.dangerous);
}

#[tokio::test]
async fn test_ollama_connection_refused() {
    let _server = MockServer::start().await;
    let config = make_local_config(LocalProviderType::Ollama, "http://127.0.0.1:1");
    let client = LocalClient::new(config);

    let messages = vec![Message {
        role: Role::User,
        content: "test".to_string(),
    }];

    let result = client.generate_command(&messages, None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_lmstudio_request_and_response() {
    let server = MockServer::start().await;
    let base_url = server.uri().to_string();
    let config = make_local_config(LocalProviderType::LmStudio, &base_url);
    let client = LocalClient::new(config);

    let content = serde_json::to_string(&json!({
        "command": "pwd",
        "explanation": "print working directory",
        "dangerous": false
    })).unwrap();

    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "choices": [{"message": {"content": content}}]
        })))
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
async fn test_ollama_list_models() {
    let server = MockServer::start().await;
    let base_url = server.uri().to_string();

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "models": [
                {"name": "llama3"},
                {"name": "phi3"}
            ]
        })))
        .mount(&server)
        .await;

    let models = LocalClient::list_models(&LocalProviderType::Ollama, &base_url)
        .await
        .unwrap();

    assert_eq!(models.len(), 2);
    assert!(models.contains(&"llama3".to_string()));
}

#[tokio::test]
async fn test_lmstudio_list_models() {
    let server = MockServer::start().await;
    let base_url = server.uri().to_string();

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": [
                {"id": "model-a"},
                {"id": "model-b"}
            ]
        })))
        .mount(&server)
        .await;

    let models = LocalClient::list_models(&LocalProviderType::LmStudio, &base_url)
        .await
        .unwrap();

    assert_eq!(models.len(), 2);
    assert!(models.contains(&"model-a".to_string()));
}
