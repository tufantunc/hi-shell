mod mock_backend;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use hi_shell::command::execute_command;
use hi_shell::llm::{CommandResponse, LlmBackend, Role};
use hi_shell::service::CommandService;

use mock_backend::{MockBackend, MockInteraction};

type ExecResult = std::result::Result<(String, bool), Box<dyn std::error::Error + Send + Sync>>;

fn success_execute(cmd: &str) -> ExecResult {
    Ok(execute_command(cmd)?)
}

fn failing_execute(_cmd: &str) -> ExecResult {
    Ok(("error: something went wrong".to_string(), false))
}

#[tokio::test]
async fn test_mock_backend_returns_command() {
    let backend = MockBackend::single("echo hello", "say hello", false);
    let result = backend.generate_command(&[], None).await.unwrap();
    assert_eq!(result.command, "echo hello");
}

#[tokio::test]
async fn test_service_simple_flow() {
    let backend = MockBackend::single("echo hello", "say hello", false);
    let interaction = MockInteraction::allow_all();
    let mut service = CommandService::new();

    let result = service
        .process_request("say hello", &backend, &interaction, success_execute)
        .await
        .unwrap();

    assert!(result.success);
    assert_eq!(result.response.command, "echo hello");
    assert!(!result.was_repair);
}

#[tokio::test]
async fn test_repair_flow_success() {
    let backend = MockBackend::new(vec![
        CommandResponse {
            command: "failing-command".to_string(),
            explanation: Some("will fail".to_string()),
            dangerous: false,
        },
        CommandResponse {
            command: "echo fixed".to_string(),
            explanation: Some("fixed version".to_string()),
            dangerous: false,
        },
    ]);

    let call_count = Arc::new(AtomicUsize::new(0));
    let interaction = MockInteraction::new(false, true);
    let mut service = CommandService::new();

    let count_clone = call_count.clone();
    let result = service
        .process_request(
            "do something",
            &backend,
            &interaction,
            move |cmd| -> ExecResult {
                let n = count_clone.fetch_add(1, Ordering::SeqCst);
                if n == 0 {
                    Ok(("command not found".to_string(), false))
                } else {
                    Ok(execute_command(cmd)?)
                }
            },
        )
        .await
        .unwrap();

    assert!(result.success);
    assert!(result.was_repair);
    assert_eq!(backend.call_count(), 2);
}

#[tokio::test]
async fn test_dangerous_command_rejected() {
    let backend = MockBackend::single("rm -rf /", "wipe disk", true);
    let interaction = MockInteraction::new(false, true);
    let mut service = CommandService::new();

    let result = service
        .process_request("wipe disk", &backend, &interaction, |cmd| {
            Ok(execute_command(cmd)?)
        })
        .await
        .unwrap();

    assert!(!result.success);
    assert!(result.output.is_empty());
}

#[tokio::test]
async fn test_dangerous_command_accepted() {
    let backend = MockBackend::single("rm -rf /tmp/test", "clean temp", true);
    let interaction = MockInteraction::new(true, true);
    let mut service = CommandService::new();

    let result = service
        .process_request("clean temp", &backend, &interaction, success_execute)
        .await
        .unwrap();

    assert!(result.success);
}

#[tokio::test]
async fn test_history_maintained() {
    let backend = MockBackend::single("echo hello", "say hello", false);
    let interaction = MockInteraction::allow_all();
    let mut service = CommandService::new();

    service
        .process_request("say hello", &backend, &interaction, success_execute)
        .await
        .unwrap();

    let history = service.history();
    assert_eq!(history.len(), 3);
    assert!(matches!(history[0].role, Role::User));
    assert!(matches!(history[1].role, Role::Assistant));
    assert!(matches!(history[2].role, Role::System));
    assert_eq!(history[0].content, "say hello");
}

#[tokio::test]
async fn test_history_truncated_at_30() {
    let interaction = MockInteraction::allow_all();
    let mut service = CommandService::new();

    for i in 0..15 {
        let backend = MockBackend::single("echo hi", "say hi", false);
        service
            .process_request(
                &format!("request {}", i),
                &backend,
                &interaction,
                success_execute,
            )
            .await
            .unwrap();
    }

    assert!(service.history().len() <= 30);
}

#[tokio::test]
async fn test_repair_flow_user_gives_up() {
    let backend = MockBackend::single("failing-command", "will always fail", false);
    let interaction = MockInteraction::new(false, false);
    let mut service = CommandService::new();

    let result = service
        .process_request("do something", &backend, &interaction, failing_execute)
        .await
        .unwrap();

    assert!(!result.success);
    assert!(!result.was_repair);
    assert_eq!(backend.call_count(), 1);
}
